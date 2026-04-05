use std::sync::LazyLock;

pub const BANNER_V1: &str = r#"
██████╗ ██████╗ ███╗   ██╗████████╗███████╗██╗  ██╗████████╗   ██████╗  ██████╗ ██╗  ██╗
██╔════╝██╔═══██╗████╗  ██║╚══██╔══╝██╔════╝╚██╗██╔╝╚══██╔══╝   ██╔══██╗██╔═══██╗╚██╗██╔╝
██║     ██║   ██║██╔██╗ ██║   ██║   █████╗   ╚███╔╝    ██║█████╗██████╔╝██║   ██║ ╚███╔╝ 
██║     ██║   ██║██║╚██╗██║   ██║   ██╔══╝   ██╔██╗    ██║╚════╝██╔══██╗██║   ██║ ██╔██╗ 
╚██████╗╚██████╔╝██║ ╚████║   ██║   ███████╗██╔╝ ██╗   ██║      ██████╔╝╚██████╔╝██╔╝ ██╗
 ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝   ╚═╝   ╚══════╝╚═╝  ╚═╝   ╚═╝      ╚═════╝  ╚═════╝ ╚═╝  ╚═╝
"#;

pub const BANNER_V2: &str = r#"
      █████████╗███████╗███╗   ███╗██████╗  ██████╗ ██╗    ██╗███╗   ██╗███████╗
      ╚══██╔══╝██╔════╝████╗ ████║██╔══██╗██╔═══██╗██║    ██║████╗  ██║██╔════╝
         ██║   █████╗  ██╔████╔██║██████╔╝██║   ██║██║ █╗ ██║██╔██╗ ██║█████╗  
         ██║   ██╔══╝  ██║╚██╔╝██║██╔══██╗██║   ██║██║███╗██║██║╚██╗██║██╔══╝  
         ██║   ███████╗██║ ╚═╝ ██║██║  ██║╚██████╔╝╚██�╚███║██║ ╚████║███████╗
         ╚═╝   ╚══════╝╚═╝     ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚═╝ ╚═══╝╚═╝  ╚═══╝╚══════╝
"#;

pub const BANNER_V3: &str = r#"
 █████╗ ███████╗ ██████╗ ███╗   ██╗    ██╗    ██╗███████╗██████╗ 
██╔══██╗██╔════╝██╔═══██╗████╗  ██║    ██║    ██║██╔════╝██╔══██╗
███████║█████╗  ██║   ██║██╔██╗ ██║    ██║ █╗ ██║█████╗  ██████╔╝
██╔══██║██╔══╝  ██║   ██║██║╚██╗██║    ██║███╗██║██╔══╝  ██╔══██╗
██║  ██║███████╗╚██████╔╝██║ ╚████║    ╚███╔███╔╝███████╗██║  ██║
╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═══╝     ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝
"#;

pub const SUBTITLE: &str = r#"
                                     ░██                             ░██            ░██                              
                                     ░██                             ░██            ░██                              
  ░███████   ░███████  ░████████  ░████████  ░███████  ░██    ░██ ░████████         ░████████   ░███████  ░██    ░██ 
 ░██    ░██ ░██    ░██ ░██    ░██    ░██    ░██    ░██  ░██  ░██     ░██    ░██████ ░██    ░██ ░██    ░██  ░██  ░██  
 ░██        ░██    ░██ ░██    ░██    ░██    ░█████████   ░█████      ░██            ░██    ░██ ░██    ░██   ░█████   
 ░██    ░██ ░██    ░██ ░██    ░██    ░██    ░██         ░██  ░██     ░██            ░███   ░██ ░██    ░██  ░██  ░██  
  ░███████   ░███████  ░██    ░██     ░████  ░███████  ░██    ░██     ░████         ░██░█████   ░███████  ░██    ░██ 
"#;

pub const BANNER_MINIMAL: &str = "ContextBox - Self-hosted Document AI Platform";

pub static BANNERS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    vec![BANNER_V1, BANNER_V2, BANNER_V3]
});

pub fn get_banner(index: usize) -> &'static str {
    BANNERS[index % BANNERS.len()]
}

pub fn get_random_banner() -> &'static str {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;
    BANNERS[seed % BANNERS.len()]
}

pub fn print_banner() {
    println!("{}", get_random_banner());
}

pub fn print_banner_index(index: usize) {
    println!("{}", get_banner(index));
}

pub fn print_subtitle() {
    println!("{}", SUBTITLE);
}

pub fn print_minimal() {
    println!("{}", BANNER_MINIMAL);
}

pub fn banner_count() -> usize {
    BANNERS.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_banners_not_empty() {
        assert!(!BANNER_V1.is_empty());
        assert!(!BANNER_V2.is_empty());
        assert!(!BANNER_V3.is_empty());
    }
    
    #[test]
    fn test_get_banner() {
        assert!(!get_banner(0).is_empty());
        assert!(!get_banner(1).is_empty());
        assert!(!get_banner(5).is_empty());
    }
    
    #[test]
    fn test_banner_count() {
        assert_eq!(banner_count(), 3);
    }
}
