const text = document.getElementById("text");

function updateText() {
  var t = text.value;
  var len = t.length;
  var pos = text.selectionStart;
  t = t.toLowerCase();

  // compound vowels
  t = t.replace("ᛟa", "ᚩ");
  t = t.replace("ᛟ.a", "ᛟᚫ");
  t = t.replace("ᛟh", "ᚩ");
  t = t.replace("ᛟ.h", "ᛟᚻ");
  t = t.replace("ᛖe", "ᛁᛁ");
  t = t.replace("ᛖ.e", "ᛖᛖ");
  t = t.replace("ᚫa", "ᚪ");
  t = t.replace("ᚫ.a", "ᚫᚫ");
  t = t.replace("ᚫu", "ᛟ");
  t = t.replace("ᚫ.u", "ᚫᚢ");
  t = t.replace("ᚢu", "ᚣ");
  t = t.replace("ᚢ.u", "ᚢᚢ");
  t = t.replace("ᛟo", "ᚣ");
  t = t.replace("ᛟ.o", "ᛟᛟ");
  t = t.replace("ᛟu", "ᚪᚹ");
  t = t.replace("ᛟ.u", "ᛟᚢ");
  t = t.replace("ᛁi", "ᛡ");
  t = t.replace("ᛁ.i", "ᛁᛁ");
  t = t.replace("ᚫi", "ᛠ");
  t = t.replace("ᚫ.i", "ᚫᛄ");
  t = t.replace("ᚫy", "ᛠ");
  t = t.replace("ᚫ.y", "ᚫᛄ");
  t = t.replace("ᛁᛁr", "ᛁᛁᚱ");
  t = t.replace("ᛁr", "ᚢᚱ");
  t = t.replace("ᛁ.r", "ᛁᚱ");
  t = t.replace("ᛟi", "ᚩᛁ");
  t = t.replace("ᛟ.i", "ᛟᛁ");
  t = t.replace("ᛟy", "ᚩᛁ");
  t = t.replace("ᛟ.y", "ᛟᛄ");
  t = t.replace("ᛖr", "ᚢᚱ");
  t = t.replace("ᛖ.r", "ᛖᚱ");
  t = t.replace("ᚫr", "ᚪᚱ");
  t = t.replace("ᚫ.r", "ᚫᚱ");
  t = t.replace("ᛟr", "ᚪᚱ");
  t = t.replace("ᛟ.r", "ᛟᚱ");
  t = t.replace("ᛟw", "ᚪᚹ");
  t = t.replace("ᛟ.w", "ᛟᚹ");
  t = t.replace("ᛢu", "ᛢ");
  t = t.replace("ᛢ.u", "ᛢᚢ");

  // compound consonants
  t = t.replace("ᛏh", "ᚦ");
  t = t.replace("ᛏ.h", "ᛏᚻ");
  t = t.replace("ᚾg", "ᛝ");
  t = t.replace("ᚾ.g", "ᚾᚷ");
  t = t.replace("ᚾk", "ᛝᚳ");
  t = t.replace("ᚾ.k", "ᚾᚳ");
  t = t.replace("ᛋt", "ᛥ");
  t = t.replace("ᛋ.t", "ᛋᛏ");

  // final short vowels
  const PUNCT = ['. ',',',':',';','!','?',')']; // period requires space after to confirm it's not used to split a replacement
  for (const punct of PUNCT) {
    t = t.replaceAll(punct, ' '+punct);
  }
  t = t.replaceAll("ᚫ ", "ᚪ ");
  t = t.replaceAll("ᛁᛁ ", "ᛁ "); // /i/ simplified word-finally
  t = t.replaceAll("ᛟ ", "ᚩ ");
  for (const punct of PUNCT) {
    t = t.replaceAll(' '+punct, punct);
  }

  // basic
  t = t.replace("q", "ᛢ");
  t = t.replace("w", "ᚹ");
  t = t.replace("e", "ᛖ");
  t = t.replace("r", "ᚱ");
  t = t.replace("t", "ᛏ");
  t = t.replace("y", "ᛄ");
  t = t.replace("u", "ᚢ");
  t = t.replace("i", "ᛁ");
  t = t.replace("o", "ᛟ");
  t = t.replace("p", "ᛈ");
  t = t.replace("a", "ᚫ");
  t = t.replace("s", "ᛋ");
  t = t.replace("d", "ᛞ");
  t = t.replace("f", "ᚠ");
  t = t.replace("g", "ᚷ");
  t = t.replace("h", "ᚻ");
  t = t.replace("j", "ᚷᚻ");
  t = t.replace("k", "ᚳ");
  t = t.replace("l", "ᛚ");
  t = t.replace("z", "ᛋ");
  t = t.replace("x", "ᛉ");
  t = t.replace("c", "ᚳ");
  t = t.replace("v", "ᚠ");
  t = t.replace("b", "ᛒ");
  t = t.replace("n", "ᚾ");
  t = t.replace("m", "ᛗ");

  // other
  t = t.replace("&", "⁊");

  if (document.settings.space.checked) {
    t = t.replaceAll(" ", "᛫");
  }
  for (const punct of PUNCT) {
    t = t.replaceAll(punct+"᛫", punct+" ");
  }
  t = t.replaceAll(".᛫",". ");
  t = t.replaceAll("᛫(", " (");

  text.value = t;
  pos += t.length - len;
  text.selectionStart = pos;
  text.selectionEnd = pos;
}
text.oninput = updateText;

function copyText() {
  text.select();
  text.setSelectionRange(0, 99999);
  document.execCommand("copy");
  //alert("Copied text to clipboard\nᚳᛟᛈᛁᛁᛞ᛫ᛏᛖᛉᛏ᛫ᛏᚣ᛫ᚳᛚᛁᛈᛒᚩᚱᛞ");
  text.setSelectionRange(0, 0);
}
function clearText() {
  if (confirm("Clear text?\nᚳᛚᛁᛁᚱ᛫ᛏᛖᛉᛏ?")) {
    text.value = "";
  }
}