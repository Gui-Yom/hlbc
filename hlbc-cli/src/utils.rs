// TODO refactor this
pub fn read_range(arg: &str, max_bound: usize) -> anyhow::Result<Box<dyn Iterator<Item = usize>>> {
    if arg == ".." {
        Ok(Box::new((0..max_bound).into_iter()))
    } else if arg.contains("..=") {
        let mut nums = arg.split("..=");
        if let Some(a) = nums.next() {
            if let Some(b) = nums.next() {
                Ok(Box::new((a.parse()?..=b.parse()?).into_iter()))
            } else if arg.ends_with(a) {
                Ok(Box::new((0..=a.parse()?).into_iter()))
            } else {
                anyhow::bail!("Inclusive range must be bounded at the end : '{arg}'")
            }
        } else {
            anyhow::bail!("Inclusive range must be bounded at the end : '{arg}'")
        }
    } else if arg.contains("..") {
        let mut nums = arg.split("..");
        if let Some(a) = nums.next() {
            if a.is_empty() {
                if let Some(b) = nums.next() {
                    Ok(Box::new((0..b.parse()?).into_iter()))
                } else {
                    anyhow::bail!("Invalid range : '{arg}'")
                }
            } else {
                if let Some(b) = nums.next() {
                    if b.is_empty() {
                        Ok(Box::new((a.parse()?..max_bound - 1).into_iter()))
                    } else {
                        Ok(Box::new((a.parse()?..b.parse()?).into_iter()))
                    }
                } else if arg.ends_with(a) {
                    Ok(Box::new((0..a.parse()?).into_iter()))
                } else if arg.starts_with(a) {
                    Ok(Box::new((a.parse()?..max_bound - 1).into_iter()))
                } else {
                    anyhow::bail!("Invalid range : '{arg}'")
                }
            }
        } else {
            Ok(Box::new((0..max_bound - 1).into_iter()))
        }
    } else {
        let i = arg.parse()?;
        Ok(Box::new((i..(i + 1)).into_iter()))
    }
}
