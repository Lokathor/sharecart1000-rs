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
  /// The high bits are ignored when saving the data.
  pub map_x: u16,

  /// This should be 0-1023 (10 bits).
  ///
  /// The high bits are ignored when saving the data.
  pub map_y: u16,

  /// Misc data.
  ///
  /// Each of these can be any bit pattern at all.
  pub misc: [u16; 4],

  /// The player's name, or something like it.
  ///
  /// When saving, any `\r` and `\n` characters are filtered away, and also only
  /// the first 1023 _bytes_ are preserved. If this results in a partial byte
  /// sequence at the end of the string it will also be stripped (so that the
  /// results are always valid utf-8).
  pub player_name: String,

  /// The eight switches.
  pub switch: [bool; 8],
}

impl Sharecart {
  /// Parses the string given into a `Sharecart` value.
  ///
  /// If the parsing works at all, then you'll get a `Sharecart` of some sort
  /// back. If any individual field is missing, then you'll get the `Default`
  /// value in that field. If the "[Main]" section is missing then you'll get
  /// the default value in every field.
  ///
  /// If the parsing fails you'll get an error message about that instead.
  ///
  /// ```rust
  /// use sharecart1000::Sharecart;
  ///
  /// let mut sc = Sharecart::default();
  /// assert_eq!(sc, Sharecart::from_str("").unwrap());
  /// assert_eq!(sc, Sharecart::from_str("[Main]").unwrap());
  ///
  /// sc.map_x = 73;
  /// sc.map_y = 1023;
  /// assert_eq!(sc, Sharecart::from_str(r#"[Main]
  /// MapX=73
  /// MapY=1023"#).unwrap());
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
  ///
  /// assert_eq!(sc, Sharecart::from_str(sc.to_string()).unwrap());
  /// ```
  pub fn from_str<S: AsRef<str>>(buf: S) -> Result<Self, String> {
    let buf_str = buf.as_ref();
    let true_enough = |v: &str| v == "TRUE" || v == "True" || v == "true";
    match ini::Ini::load_from_str(buf_str) {
      Err(ini::ini::Error { msg, .. }) => Err(msg),
      Ok(i) => match i.section(Some("Main")).or(i.section(Some("main"))) {
        Some(properties) => {
          let mut sc = Sharecart::default();
          for (k, v) in properties.iter() {
            let k: &str = k.as_ref();
            match k {
              "MapX" => {
                sc.map_x = v.parse::<u16>().unwrap_or(0) % 1024;
              }
              "MapY" => {
                sc.map_y = v.parse::<u16>().unwrap_or(0) % 1024;
              }
              "Misc0" => {
                sc.misc[0] = v.parse::<u16>().unwrap_or(0);
              }
              "Misc1" => {
                sc.misc[1] = v.parse::<u16>().unwrap_or(0);
              }
              "Misc2" => {
                sc.misc[2] = v.parse::<u16>().unwrap_or(0);
              }
              "Misc3" => {
                sc.misc[3] = v.parse::<u16>().unwrap_or(0);
              }
              "PlayerName" => {
                sc.player_name = v.to_string();
              }
              "Switch0" => {
                sc.switch[0] = true_enough(v);
              }
              "Switch1" => {
                sc.switch[1] = true_enough(v);
              }
              "Switch2" => {
                sc.switch[2] = true_enough(v);
              }
              "Switch3" => {
                sc.switch[3] = true_enough(v);
              }
              "Switch4" => {
                sc.switch[4] = true_enough(v);
              }
              "Switch5" => {
                sc.switch[5] = true_enough(v);
              }
              "Switch6" => {
                sc.switch[6] = true_enough(v);
              }
              "Switch7" => {
                sc.switch[7] = true_enough(v);
              }
              _ => {}
            }
          }
          Ok(sc)
        }
        None => Ok(Sharecart::default()),
      },
    }
  }

  /// Gives you a `String` that you can write into the `o_o.ini` file.
  ///
  /// The string includes the "[Main]" section tag and other proper `ini`
  /// formatting, so that you can completely replace the current `o_o.ini`
  /// contents with this new string when saving the game.
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
    let byte_vec: Vec<u8> = self.player_name.bytes().filter(|&b| b != b'\r' && b != b'\n').take(1023).collect();
    for ch in String::from_utf8_lossy(&byte_vec).chars() {
      if ch == '�' {
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

  assert_eq!(Sharecart::default(), Sharecart::from_str(sc.to_string()).unwrap());
}

#[test]
fn test_sharecart_player_name_safe() {
  let mut sc = Sharecart::default();

  sc.player_name = "\r\n".to_string();
  assert_eq!(Sharecart::default(), Sharecart::from_str(sc.to_string()).unwrap());

  sc.player_name = "x".repeat(2_000);
  let round_trip = Sharecart::from_str(sc.to_string()).unwrap();
  assert_eq!(round_trip.player_name.len(), 1023);
}
