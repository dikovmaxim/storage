use anyhow::{bail, Context, Result};
use libc::{c_ulong, ioctl, ENOTTY, EOPNOTSUPP};
use nix::fcntl::{open, OFlag};
use nix::sys::socket::{socketpair, AddressFamily, SockFlag, SockType};
use nix::sys::stat::Mode;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::Mutex;

// ===== Linux UAPI: include/uapi/linux/nbd.h =====
// _IO(0xab, X)
// Linux _IOC encoding:
// dir(2) | type(8) | nr(8) | size(14)
// _IO => dir=0, size=0
const fn ioc_none(group: u8, num: u8) -> c_ulong {
    ((group as c_ulong) << 8) | (num as c_ulong)
}

const NBD_SET_SOCK: c_ulong    = ioc_none(0xab, 0);
const NBD_SET_BLKSIZE: c_ulong = ioc_none(0xab, 1);
const NBD_SET_SIZE: c_ulong    = ioc_none(0xab, 2);
const NBD_DO_IT: c_ulong       = ioc_none(0xab, 3);
const NBD_CLEAR_SOCK: c_ulong  = ioc_none(0xab, 4);
const NBD_CLEAR_QUE: c_ulong   = ioc_none(0xab, 5);

// NBD protocol magics
const NBD_REQUEST_MAGIC: u32 = 0x2560_9513;
const NBD_REPLY_MAGIC: u32 = 0x6744_6698;

// Command types (subset; kernel may send others if you set flags)
const NBD_CMD_READ: u32 = 0;
const NBD_CMD_WRITE: u32 = 1;
const NBD_CMD_DISC: u32 = 2;
const NBD_CMD_FLUSH: u32 = 3;
// others exist (TRIM, WRITE_ZEROES, etc). We’ll return EOPNOTSUPP.

#[derive(Debug)]
struct Req {
    cmd: u32,
    handle: [u8; 8],
    offset: u64,
    len: u32,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {

    let dev_path = "/dev/nbd0";
    let size_mib: u64 = 512; // 512 MiB
    let size_bytes = size_mib * 1024 * 1024;
    let blksize: u64 = 4096;

    if size_bytes % blksize != 0 {
        bail!("size must be multiple of {}", blksize);
    }

    // backing store
    let store = Arc::new(Mutex::new(vec![0u8; size_bytes as usize]));

    // open /dev/nbdX
    let nbd_fd = open(dev_path, OFlag::O_RDWR, Mode::empty()).context("open nbd dev")?;
    let nbd_raw = nbd_fd.as_raw_fd();

    // socketpair kernel<->userspace
    let (k_sock, u_sock) = socketpair(
        AddressFamily::Unix,
        SockType::Stream,
        None,
        SockFlag::empty(),
    )
    .context("socketpair")?;

    // configure NBD
    unsafe {
        if ioctl(nbd_raw, NBD_SET_BLKSIZE, blksize as c_ulong) != 0 {
            bail!("NBD_SET_BLKSIZE: {}", std::io::Error::last_os_error());
        }
        if ioctl(nbd_raw, NBD_SET_SIZE, size_bytes as c_ulong) != 0 {
            bail!("NBD_SET_SIZE: {}", std::io::Error::last_os_error());
        }
        if ioctl(nbd_raw, NBD_SET_SOCK, k_sock.as_raw_fd() as c_ulong) != 0 {
            bail!("NBD_SET_SOCK: {}", std::io::Error::last_os_error());
        }
    }

    // NBD_DO_IT blocks; run it in a dedicated thread.
    // Keep the fd alive inside the thread.
    let nbd_fd_for_thread = nbd_fd;
    let do_it = std::thread::spawn(move || {
        // This blocks until disconnect.
        let fd = nbd_fd_for_thread.as_raw_fd();
        unsafe {
            let r = ioctl(fd, NBD_DO_IT, 0);
            // best-effort cleanup
            let _ = ioctl(fd, NBD_CLEAR_QUE, 0);
            let _ = ioctl(fd, NBD_CLEAR_SOCK, 0);
            r
        }
    });

    // Wrap the userspace end in Tokio
    let user_stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(u_sock.as_raw_fd()) };
    user_stream
        .set_nonblocking(true)
        .context("set_nonblocking")?;
    // IMPORTANT: we “forget” u_sock so it doesn’t close the fd we just moved
    std::mem::forget(u_sock);

    let mut io = UnixStream::from_std(user_stream).context("tokio UnixStream")?;

    eprintln!(
        "attached {} ({} MiB). In another shell: mkfs.ext4 {} && mount {} /mnt",
        dev_path, size_mib, dev_path, dev_path
    );

    // main request loop
    loop {
        let req = match read_req(&mut io).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("read_req ended: {e:?}");
                break;
            }
        };

        if req.cmd == NBD_CMD_DISC {
            eprintln!("got DISC");
            // reply is not required for DISC in many setups; we just break
            break;
        }

        match req.cmd {
            NBD_CMD_READ => {
                let data = {
                    let s = store.lock().await;
                    let off = req.offset as usize;
                    let len = req.len as usize;
                    if off + len > s.len() {
                        // EINVAL
                        (libc::EINVAL as u32, Vec::new())
                    } else {
                        (0u32, s[off..off + len].to_vec())
                    }
                };
                println!("READ @{} len {} => err {}", req.offset, req.len, data.0);
                write_reply(&mut io, req.handle, data.0, Some(&data.1)).await?;
            }
            NBD_CMD_WRITE => {
                let mut buf = vec![0u8; req.len as usize];
                io.read_exact(&mut buf).await.context("read write payload")?;

                let err = {
                    let mut s = store.lock().await;
                    let off = req.offset as usize;
                    if off + buf.len() > s.len() {
                        libc::EINVAL as u32
                    } else {
                        s[off..off + buf.len()].copy_from_slice(&buf);
                        0u32
                    }
                };
                println!("WRITE @{} len {} => err {}", req.offset, req.len, err);
                write_reply(&mut io, req.handle, err, None).await?;
            }
            NBD_CMD_FLUSH => {
                // our backing store is memory; nothing to do
                println!("FLUSH");
                write_reply(&mut io, req.handle, 0, None).await?;
            }
            _ => {
                // This is where TRIM / WRITE_ZEROES would land if the kernel sends them.
                // “Advertise not implemented”: don’t set the flags in the ioctl handshake.
                // If you still receive it, return EOPNOTSUPP.
                write_reply(&mut io, req.handle, EOPNOTSUPP as u32, None).await?;
            }
        }
    }

    // dropping the user socket should make NBD_DO_IT return
    drop(io);
    let _ = do_it.join();

    Ok(())
}

async fn read_req(io: &mut UnixStream) -> Result<Req> {
    // nbd_request is 28 bytes packed
    // __be32 magic; __be32 type; char handle[8]; __be64 from; __be32 len;
    let mut hdr = [0u8; 28];
    io.read_exact(&mut hdr).await.context("read request hdr")?;

    let magic = u32::from_be_bytes(hdr[0..4].try_into().unwrap());
    if magic != NBD_REQUEST_MAGIC {
        bail!("bad request magic: {:#x}", magic);
    }

    let cmd = u32::from_be_bytes(hdr[4..8].try_into().unwrap());
    let mut handle = [0u8; 8];
    handle.copy_from_slice(&hdr[8..16]);
    let offset = u64::from_be_bytes(hdr[16..24].try_into().unwrap());
    let len = u32::from_be_bytes(hdr[24..28].try_into().unwrap());

    Ok(Req {
        cmd,
        handle,
        offset,
        len,
    })
}

async fn write_reply(io: &mut UnixStream, handle: [u8; 8], err: u32, data: Option<&[u8]>) -> Result<()> {
    // nbd_reply: __be32 magic; __be32 error; char handle[8];
    let mut rep = [0u8; 16];
    rep[0..4].copy_from_slice(&NBD_REPLY_MAGIC.to_be_bytes());
    rep[4..8].copy_from_slice(&err.to_be_bytes());
    rep[8..16].copy_from_slice(&handle);

    io.write_all(&rep).await.context("write reply hdr")?;
    if let Some(d) = data {
        io.write_all(d).await.context("write reply data")?;
    }
    Ok(())
}
