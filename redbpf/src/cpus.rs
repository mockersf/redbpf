use std::fs::read;
use std::io::Error;
use std::str::FromStr;

const SYS_CPU_ONLINE: &str = "/sys/devices/system/cpu/online";

pub type CpuId = i32;

/// Returns a list of online CPU IDs.
///
/// Error handling in this function is deliberately crashy
/// If the kernel returns with invalid data, it's OK to crash
/// If the kernel returns with different data format, it's OK to crash
/// If the user is trying to feed us invalid data, it's OK to crash
///
/// The only time an error is reported is when
/// `/sys/devices/system/cpu/online` can't be opened.
pub fn get_online() -> Result<Vec<CpuId>, Error> {
    let cpus = unsafe { String::from_utf8_unchecked(read(SYS_CPU_ONLINE)?) };
    Ok(list_from_string(&cpus.trim()))
}

fn list_from_string(cpus: &str) -> Vec<CpuId> {
    let cpu_list = cpus.split(',').flat_map(|group| {
        let mut split = group.split('-');
        let start = split.next();
        let end = split.next();

        if let (Some(start), None) = (start, end) {
            let cpuid = CpuId::from_str(start).unwrap();
            return cpuid..=cpuid;
        }

        let start = CpuId::from_str(start.unwrap()).unwrap();
        let end = CpuId::from_str(end.unwrap()).unwrap();
        (start..=end)
    });
    cpu_list.collect()
}

mod test {
    #[test]
    fn test() {
        use crate::cpus::list_from_string;
        assert_eq!(list_from_string("0"), vec![0]);
        assert_eq!(list_from_string("0-4"), vec![0, 1, 2, 3, 4]);
        assert_eq!(list_from_string("0-2,5-6"), vec![0, 1, 2, 5, 6]);
    }
}
