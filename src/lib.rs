//! This crate lets you interact with the
//! [Sharecart1000](http://sharecart1000.com/) system.
//!
//! The format specification is given
//! [here](http://sharecart1000.com/img/SHARECART1000guide.png).
//!
//! See their [about page](http://alts.github.io/sharecart.lua/) for more info.

#![forbid(missing_debug_implementations)]
#![forbid(missing_docs)]
#![forbid(unsafe_code)]

extern crate ini;

/// This is your Sharecart data, in a rusty form.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sharecart {
  /// This should be 0-1023 (10 bits).
  ///
  /// * Saving: Ignores the high bits (eg: `cart.map_x % 1024`)
  /// * Loading: Attempts to parse a `u16` (0 on failure) and then truncates to
  ///   10 bits.
  pub map_x: u16,

  /// This should be 0-1023 (10 bits).
  ///
  /// * Saving: Ignores the high bits (eg: `cart.map_y % 1024`)
  /// * Loading: Attempts to parse a `u16` (0 on failure) and then truncates to
  ///   10 bits.
  pub map_y: u16,

  /// Misc data.
  ///
  /// * Saving: The full range is supported
  /// * Loading: Attempts to parse a `u16`, 0 on failure.
  pub misc: [u16; 4],

  /// The player's name, or something like it.
  ///
  /// The definition of "1023chars" is slightly fuzzy when you get into the fact
  /// that there's multi-byte characters, but that some languages assume all
  /// chars are 1 byte. While we're working with it in memory, we just act like
  /// it's a normal `String` value. If you're just using this field for 1,023 or
  /// fewer ASCII characters (without line endings), you'll be totally fine.
  /// Otherwise there's some edge cases to worry about.
  ///
  /// * Saving: Takes the first 1023 _bytes_, then lossy re-parses the bytes as
  ///   chars and filters out any `'\u{0FFFD}'`, `'\r'`, and `'\n'`.
  /// * Loading: Performs a similar contortion, where the first 1023 bytes are
  ///   taken, lossy parsed for utf8, filtered, and then that result is kept.
  pub player_name: String,

  /// The eight switches.
  ///
  /// * Saving: Always outputs as "TRUE" or "FALSE".
  /// * Loading: Ignores case, so that "True" and "TrUe" and such are also
  ///   allowed as `true`. Any value that isn't read as `true` becomes `false`.
  pub switch: [bool; 8],
}

impl Sharecart {
  /// Parses the string given into a `Sharecart` value.
  ///
  /// You will always get a `Sharecart` of some sort back. If any individual
  /// field is missing, then you'll get the `Default` value in that field. If
  /// the "[Main]" section is missing then you'll get the default value in every
  /// field. If the string somehow can't even be parsed at all then you'll get
  /// the default value in every field. Field names ignore capitalization
  /// differences.
  ///
  /// ```rust
  /// use sharecart1000::Sharecart;
  ///
  /// let mut sc = Sharecart::default();
  /// assert_eq!(sc, Sharecart::from_str(""));
  /// assert_eq!(sc, Sharecart::from_str("[Main]"));
  ///
  /// sc.map_x = 73;
  /// sc.map_y = 1023;
  /// assert_eq!(sc, Sharecart::from_str(r#"[Main]
  /// MapX=73
  /// MapY=1023"#));
  ///
  /// sc.misc[0] = 54;
  /// sc.misc[1] = 540;
  /// sc.misc[2] = 999;
  /// sc.misc[3] = ::std::u16::MAX;
  /// sc.player_name = "Fearless Concurrency".to_string();
  /// let mut foo = true;
  /// for i in 0 .. 8 {
  ///   sc.switch[i] = foo;
  ///   foo = !foo;
  /// }
  /// assert_eq!(sc, Sharecart::from_str(sc.to_string()));
  /// ```
  pub fn from_str<S: AsRef<str>>(buf: S) -> Self {
    let buf_str = buf.as_ref();
    match ini::Ini::load_from_str(buf_str) {
      Ok(i) => match i.section(Some("Main")).or(i.section(Some("main"))) {
        Some(properties) => {
          let mut sc = Sharecart::default();
          for (k, v) in properties.iter() {
            let lower = k.to_lowercase();
            match lower.as_ref() {
              "mapx" => {
                sc.map_x = v.parse::<u16>().unwrap_or(0) % 1024;
              }
              "mapy" => {
                sc.map_y = v.parse::<u16>().unwrap_or(0) % 1024;
              }
              "misc0" => {
                sc.misc[0] = v.parse::<u16>().unwrap_or(0);
              }
              "misc1" => {
                sc.misc[1] = v.parse::<u16>().unwrap_or(0);
              }
              "misc2" => {
                sc.misc[2] = v.parse::<u16>().unwrap_or(0);
              }
              "misc3" => {
                sc.misc[3] = v.parse::<u16>().unwrap_or(0);
              }
              "playername" => {
                let byte_vec: Vec<u8> = v.bytes().take(1024).collect();
                sc.player_name = String::from_utf8_lossy(&byte_vec)
                  .chars()
                  .filter(|&c| c != '\u{0FFFD}' && c != '\r' && c != '\n')
                  .collect();
              }
              "switch0" => {
                sc.switch[0] = v.to_lowercase() == "true";
              }
              "switch1" => {
                sc.switch[1] = v.to_lowercase() == "true";
              }
              "switch2" => {
                sc.switch[2] = v.to_lowercase() == "true";
              }
              "switch3" => {
                sc.switch[3] = v.to_lowercase() == "true";
              }
              "switch4" => {
                sc.switch[4] = v.to_lowercase() == "true";
              }
              "switch5" => {
                sc.switch[5] = v.to_lowercase() == "true";
              }
              "switch6" => {
                sc.switch[6] = v.to_lowercase() == "true";
              }
              "switch7" => {
                sc.switch[7] = v.to_lowercase() == "true";
              }
              _ => {}
            }
          }
          sc
        }
        None => Sharecart::default(),
      },
      Err(_) => Sharecart::default(),
    }
  }

  /// Gives you a `String` that you can write into the `o_o.ini` file.
  ///
  /// The string includes the "[Main]" section tag and other proper `ini`
  /// formatting, so that you can completely replace the current `o_o.ini`
  /// contents with this new string when saving the game. Lines are always
  /// separated by just `'\n'`, which is is the most cross-platform way to
  /// handle line-endings while also being consistent.
  ///
  /// ```rust
  /// use sharecart1000::Sharecart;
  /// assert_eq!(Sharecart::default().to_string(), r#"[Main]
  /// MapX=0
  /// MapY=0
  /// Misc0=0
  /// Misc1=0
  /// Misc2=0
  /// Misc3=0
  /// PlayerName=
  /// Switch0=FALSE
  /// Switch1=FALSE
  /// Switch2=FALSE
  /// Switch3=FALSE
  /// Switch4=FALSE
  /// Switch5=FALSE
  /// Switch6=FALSE
  /// Switch7=FALSE
  /// "#);
  /// ```
  pub fn to_string(&self) -> String {
    // There's about 170 chars of just boilerplate, so we'll get more than the
    // default capacity here.
    let mut s = String::with_capacity(200);

    s.push_str("[Main]\n");
    s.push_str(&format!("MapX={}\n", self.map_x % 1024));
    s.push_str(&format!("MapY={}\n", self.map_y % 1024));
    for i in 0..4 {
      s.push_str(&format!("Misc{}={}\n", i, self.misc[i]));
    }
    s.push_str("PlayerName=");
    let byte_vec: Vec<u8> = self.player_name.bytes().take(1023).collect();
    for ch in String::from_utf8_lossy(&byte_vec).chars() {
      if ch == '\u{0FFFD}' || ch == '\r' || ch == '\n' {
        continue;
      }
      s.push(ch);
    }
    s.push('\n');
    for i in 0..8 {
      s.push_str(&format!("Switch{}={}\n", i, if self.switch[i] { "TRUE" } else { "FALSE" }));
    }

    s
  }
}

#[test]
fn test_sharecart_10bit_safe() {
  let mut sc = Sharecart::default();

  for bit in 11..16 {
    sc.map_x |= 1 << bit;
    sc.map_y |= 1 << bit;
  }

  assert_eq!(Sharecart::default(), Sharecart::from_str(sc.to_string()));
}

#[test]
fn test_sharecart_player_name_safe() {
  let mut sc = Sharecart::default();

  sc.player_name = "\r\n".to_string();
  assert_eq!(Sharecart::default(), Sharecart::from_str(sc.to_string()));

  sc.player_name = "x".repeat(2_000);
  let round_trip = Sharecart::from_str(sc.to_string());
  assert_eq!(round_trip.player_name.len(), 1023);
}
