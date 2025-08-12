#![no_std]

use core::arch::naked_asm;

use linux_syscalls::{syscall, Errno, Sysno};

#[repr(C)]
struct InitFrame {
    argc: i64,
}

impl InitFrame {
    unsafe fn get_argv<'a>(f: *const Self) -> &'a [*const u8] {
        let cnt = (*f).argc;
        core::slice::from_raw_parts(f.byte_offset(8) as *const *const u8, cnt as _)
    }
}

unsafe fn mymain(initf: *const InitFrame) -> Result<(), Errno> {
    let args = InitFrame::get_argv(initf);
    if args.len() != 3 {
        return Err(Errno::new(3));
    }
    let srcfd = syscall!([ro] Sysno::open_tree, libc::AT_FDCWD, args[1],
                         (libc::AT_NO_AUTOMOUNT|libc::AT_SYMLINK_NOFOLLOW) as u32
                         | libc::OPEN_TREE_CLONE)?;
    let empty_str = 0u8;
    syscall!([ro] Sysno::move_mount, srcfd, &empty_str as *const _, libc::AT_FDCWD, args[2],
             libc::MOVE_MOUNT_F_EMPTY_PATH)?;
    Ok(())
}

unsafe extern "C" fn entry(initf: *const InitFrame) -> ! {
    let errno = mymain(initf).err().map_or(0, |e| e.into_raw());
    syscall!([!] Sysno::exit_group, errno)
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn _start() -> ! {
    naked_asm!(
        "xor %ebp, %ebp",
        "mov %rsp, %rdi",
        "call {main}",
        main=sym entry,
        options(att_syntax)
    )
}

#[cfg(not(test))]
#[panic_handler]
unsafe fn panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::asm!("int3", options(noreturn));
}
