use chrono::naive::time::NaiveTime as ChronoNaiveTime;

use parse::Parse;

pub struct Time {
    hour:   u32,
    minute: u32,
    second: u32,
}

impl Time {

    pub fn new(hour: u32, minute: u32, second: u32) -> Time {
        Time {
            hour: hour,
            minute: minute,
            second: second
        }
    }

    pub fn hour(&self) -> u32 {
        self.hour
    }

    pub fn minute(&self) -> u32 {
        self.minute
    }

    pub fn second(&self) -> u32 {
        self.second
    }

}

impl Into<ChronoNaiveTime> for Time {

    fn into(self) -> ChronoNaiveTime {
        ChronoNaiveTime::from_hms(self.hour, self.minute, self.second)
    }

}

impl Parse for Time {

    fn parse(s: &str) -> Option<Time> {
        use std::str::FromStr;
        use regex::Regex;
        use parse::time_parse_regex;

        lazy_static! {
            static ref R: Regex = Regex::new(time_parse_regex()).unwrap();
        }

        R.captures(s)
            .and_then(|capts| {
                let hour   = capts.name("h").and_then(|o| FromStr::from_str(o).ok());
                let minute = capts.name("m").and_then(|o| FromStr::from_str(o).ok()).unwrap_or(0);
                let second = capts.name("s").and_then(|o| FromStr::from_str(o).ok()).unwrap_or(0);

                if hour.is_none() {
                    debug!("No hour");
                    return None;
                }

                Some(Time::new(hour.unwrap(), minute, second))
            })
    }

}

#[cfg(test)]
mod test {
    use super::Time;
    use parse::Parse;

    #[test]
    fn test_valid() {
        let s = "2016-12-12T20:01:02";
        let t = Time::parse(s);

        assert!(t.is_some());
        let t = t.unwrap();

        assert_eq!(20, t.hour());
        assert_eq!(1, t.minute());
        assert_eq!(2, t.second());
    }

    #[test]
    fn test_valid_without_sec() {
        let s = "2016-12-12T20:01";
        let t = Time::parse(s);

        assert!(t.is_some());
        let t = t.unwrap();

        assert_eq!(20, t.hour());
        assert_eq!(1, t.minute());
        assert_eq!(0, t.second());
    }

    #[test]
    fn test_valid_without_min() {
        let s = "2016-12-12T20";
        let t = Time::parse(s);

        assert!(t.is_some());
        let t = t.unwrap();

        assert_eq!(20, t.hour());
        assert_eq!(0, t.minute());
        assert_eq!(0, t.second());
    }

    #[test]
    fn test_invalid() {
        assert!(Time::parse("2015-12-12T").is_none());
        assert!(Time::parse("2015-12-12T200").is_none());
        assert!(Time::parse("2015-12-12T20-20").is_none());
        assert!(Time::parse("2015-12-12T20:200").is_none());
        assert!(Time::parse("2015-12-12T20:20:200").is_none());
        assert!(Time::parse("2015-12-12T20:20:").is_none());
        assert!(Time::parse("2015-12-12T20:").is_none());
        assert!(Time::parse("2015-12-12T2:20:21").is_none());
        assert!(Time::parse("2015-12-12T2:2:20").is_none());
        assert!(Time::parse("2015-12-12T2:2:2").is_none());
    }

}

