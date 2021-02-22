use game_lib::*;

pub fn load_sprites(gcontext: &mut GContext) {
  #[rustfmt::skip]
  gcontext.add_sprite(
    "ball",
    vec![
      "  ####  ",
      " #__### ",
      "#_######",
      "#_######",
      "########",
      "########",
      " ###### ",
      "  ####  ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "cherry",
    vec![
      "  /     ",
      " / /    ",
      " /  /   ",
      " /   ## ",
      " ## #_##",
      "#_## ###",
      "#### ## ",
      " ##     ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "coin",
    vec![
      "  &&&&  ",
      " &&^^^& ",
      "&&^^&^^&",
      "&&^^&^^&",
      "&&^^&^^&",
      "&&^^&^^&",
      " &&^^^& ",
      "  &&&&  ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "gin",
    vec![
      "_______ ",
      " _ccc_  ",
      "  _c_   ",
      "   _    ",
      "   _    ",
      "   _    ",
      "   _    ",
      "  ___   ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "floor",
    vec![
      " o oo oo",
      "o  o   o",
      "  o oo  ",
      "o   o  o",
      "  oo o o",
      "o  o    ",
      "    o  o",
      " oo oo  ",
    ],
  );
  #[rustfmt::skip]
  gcontext.add_sprite(
    "wall",
    vec![
      "cccccccc",
      "c  c   c",
      "cccccccc",
      "c c  c c",
      "cccccccc",
      "c   c  c",
      "c   c  c",
      "cccccccc",
    ],
  );
}
