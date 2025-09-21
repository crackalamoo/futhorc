const text = document.getElementById("text");

// Track the Latin word the user is currently typing so we can respond to
// deletions even after the visible text has been converted to runes.
let currentLatinWord = "";
let lastRunifiedValue = text.value || "";
let currentWordStartIndex = null;

const WORD_BOUNDARY_CHARS = new Set([
  " ", "\n", "\r", "\t", ".", ",", ":", ";", "!", "?", "(", ")",
  "[", "]", "{", "}", "\"", "-", "—", "᛫", "&", "⁊"
]);

const LATIN_WORD_CHAR_REGEX = /[a-z']/;

const PUNCT = ['. ',',',':',';','!','?',')'];

const RUNE_LATIN_OPTIONS = new Map([
  ["ᛢ", ["q"]],
  ["ᚹ", ["w"]],
  ["ᛖ", ["e"]],
  ["ᚱ", ["r"]],
  ["ᛏ", ["t"]],
  ["ᛄ", ["y"]],
  ["ᚢ", ["u"]],
  ["ᛁ", ["i", "e", "y"]],
  ["ᛟ", ["o", "au", "ow"]],
  ["ᛈ", ["p"]],
  ["ᚫ", ["a", "ae"]],
  ["ᛋ", ["s", "z"]],
  ["ᛞ", ["d"]],
  ["ᚠ", ["f", "v"]],
  ["ᚷ", ["g"]],
  ["ᚻ", ["h"]],
  ["ᚳ", ["c", "k"]],
  ["ᛚ", ["l"]],
  ["ᛉ", ["x"]],
  ["ᛒ", ["b"]],
  ["ᚾ", ["n"]],
  ["ᛗ", ["m"]],
  ["ᚦ", ["th"]],
  ["ᛝ", ["ng", "n"]],
  ["ᛥ", ["st"]],
  ["ᚣ", ["oo", "uu"]],
  ["ᛡ", ["ii", "ee", "ie", "yi"]],
  ["ᛠ", ["ai", "ay"]],
  ["ᚪ", ["aa", "ao", "ah", "a", "o"]],
  ["ᚩ", ["oa", "oh", "oi", "oy", "eo", "o"]],
  ["⁊", ["&"]]
]);

function isBoundaryChar(ch) {
  return WORD_BOUNDARY_CHARS.has(ch) || /\s/.test(ch);
}

function removeSuffixForOptions(options) {
  for (const suffix of options) {
    if (suffix && currentLatinWord.endsWith(suffix)) {
      currentLatinWord = currentLatinWord.slice(0, -suffix.length);
      return true;
    }
  }
  return false;
}

function findWordStartIndex(source, boundaryPosition) {
  let cursor = Math.max(0, boundaryPosition);
  while (cursor > 0) {
    const priorChar = source[cursor - 1];
    if (!priorChar || isBoundaryChar(priorChar)) {
      break;
    }
    cursor -= 1;
  }
  return cursor;
}

function handleAddition(added, startIndex, source) {
  const completed = [];
  let index = typeof startIndex === "number" ? startIndex : 0;
  for (const ch of added) {
    const isWordChar = LATIN_WORD_CHAR_REGEX.test(ch);

    if (isBoundaryChar(ch)) {
      if (currentLatinWord) {
        let start = currentWordStartIndex;
        if (typeof start !== "number") {
          start = findWordStartIndex(source ?? "", index);
        }
        completed.push({ word: currentLatinWord, boundary: ch, start, end: index });
      }
      currentLatinWord = "";
      currentWordStartIndex = null;
    } else if (isWordChar) {
      if (!currentLatinWord) {
        currentWordStartIndex = index;
      }
      currentLatinWord += ch;
    } else {
      currentLatinWord = "";
      currentWordStartIndex = null;
    }
    index += ch.length;
  }
  return completed;
}

function handleRemoval(removed) {
  let index = removed.length;
  while (index > 0) {
    const ch = removed[index - 1];
    if (isBoundaryChar(ch)) {
      currentLatinWord = "";
      currentWordStartIndex = null;
      index -= 1;
      continue;
    }
    const options = RUNE_LATIN_OPTIONS.get(ch);
    if (options && removeSuffixForOptions(options)) {
      index -= 1;
      continue;
    }
    const isWordChar = LATIN_WORD_CHAR_REGEX.test(ch);
    if (isWordChar) {
      if (currentLatinWord.endsWith(ch)) {
        currentLatinWord = currentLatinWord.slice(0, -1);
      } else if (currentLatinWord.length > 0) {
        currentLatinWord = currentLatinWord.slice(0, -1);
      }
    } else if (currentLatinWord.length > 0) {
      currentLatinWord = currentLatinWord.slice(0, -1);
    }
    index -= 1;
  }
  if (!currentLatinWord) {
    currentWordStartIndex = null;
  }
}

function diffStrings(prev, curr) {
  let prefix = 0;
  const maxPrefix = Math.min(prev.length, curr.length);
  while (prefix < maxPrefix && prev[prefix] === curr[prefix]) {
    prefix += 1;
  }

  let suffix = 0;
  const prevRemaining = prev.length - prefix;
  const currRemaining = curr.length - prefix;
  const maxSuffix = Math.min(prevRemaining, currRemaining);
  while (
    suffix < maxSuffix &&
    prev[prev.length - 1 - suffix] === curr[curr.length - 1 - suffix]
  ) {
    suffix += 1;
  }

  const removed = prev.slice(prefix, prev.length - suffix);
  const added = curr.slice(prefix, curr.length - suffix);
  return { removed, added, insertionIndex: prefix };
}

function applyConversionPass(value) {
  let t = value;

  // compound vowels
  t = t.replaceAll("ᛟa", "ᚩ");
  t = t.replaceAll("ᛟ.a", "ᛟᚫ");
  t = t.replaceAll("ᛟh", "ᚩ");
  t = t.replaceAll("ᛟ.h", "ᛟᚻ");
  t = t.replaceAll("ᛖe", "ᛁᛁ");
  t = t.replaceAll("ᛖ.e", "ᛖᛖ");
  t = t.replaceAll("ᚫa", "ᚪ");
  t = t.replaceAll("ᚫ.a", "ᚫᚫ");
  t = t.replaceAll("ᚫu", "ᛟ");
  t = t.replaceAll("ᚫ.u", "ᚫᚢ");
  t = t.replaceAll("ᚢu", "ᚣ");
  t = t.replaceAll("ᚢ.u", "ᚢᚢ");
  t = t.replaceAll("ᛟo", "ᚣ");
  t = t.replaceAll("ᛟ.o", "ᛟᛟ");
  t = t.replaceAll("ᛟu", "ᚪᚹ");
  t = t.replaceAll("ᛟ.u", "ᛟᚢ");
  t = t.replaceAll("ᛁi", "ᛡ");
  t = t.replaceAll("ᛁ.i", "ᛁᛁ");
  t = t.replaceAll("ᚫi", "ᛠ");
  t = t.replaceAll("ᚫ.i", "ᚫᛄ");
  t = t.replaceAll("ᚫy", "ᛠ");
  t = t.replaceAll("ᚫ.y", "ᚫᛄ");
  t = t.replaceAll("ᛁᛁr", "ᛁᛁᚱ");
  t = t.replaceAll("ᛁr", "ᚢᚱ");
  t = t.replaceAll("ᛁ.r", "ᛁᚱ");
  t = t.replaceAll("ᛟi", "ᚩᛁ");
  t = t.replaceAll("ᛟ.i", "ᛟᛁ");
  t = t.replaceAll("ᛟy", "ᚩᛁ");
  t = t.replaceAll("ᛟ.y", "ᛟᛄ");
  t = t.replaceAll("ᛖr", "ᚢᚱ");
  t = t.replaceAll("ᛖ.r", "ᛖᚱ");
  t = t.replaceAll("ᚫr", "ᚪᚱ");
  t = t.replaceAll("ᚫ.r", "ᚫᚱ");
  t = t.replaceAll("ᛟr", "ᚩᚱ");
  t = t.replaceAll("ᛟ.r", "ᛟᚱ");
  t = t.replaceAll("ᛟw", "ᚪᚹ");
  t = t.replaceAll("ᛟ.w", "ᛟᚹ");
  t = t.replaceAll("ᛢu", "ᛢ");
  t = t.replaceAll("ᛢ.u", "ᛢᚢ");

  // compound consonants
  t = t.replaceAll("ᛏh", "ᚦ");
  t = t.replaceAll("ᛏ.h", "ᛏᚻ");
  t = t.replaceAll("ᚾg", "ᛝ");
  t = t.replaceAll("ᚾ.g", "ᚾᚷ");
  t = t.replaceAll("ᚾk", "ᛝᚳ");
  t = t.replaceAll("ᚾ.k", "ᚾᚳ");
  t = t.replaceAll("ᛋt", "ᛥ");
  t = t.replaceAll("ᛋ.t", "ᛋᛏ");

  // final short vowels
  for (const punct of PUNCT) {
    t = t.replaceAll(punct, ' ' + punct);
  }
  t = t.replaceAll("ᚫ ", "ᚪ ");
  t = t.replaceAll("ᛁᛁ ", "ᛁ "); // /i/ simplified word-finally
  t = t.replaceAll("ᛟ ", "ᚩ ");
  for (const punct of PUNCT) {
    t = t.replaceAll(' ' + punct, punct);
  }

  // basic
  t = t.replaceAll("q", "ᛢ");
  t = t.replaceAll("w", "ᚹ");
  t = t.replaceAll("e", "ᛖ");
  t = t.replaceAll("r", "ᚱ");
  t = t.replaceAll("t", "ᛏ");
  t = t.replaceAll("y", "ᛄ");
  t = t.replaceAll("u", "ᚢ");
  t = t.replaceAll("i", "ᛁ");
  t = t.replaceAll("o", "ᛟ");
  t = t.replaceAll("p", "ᛈ");
  t = t.replaceAll("a", "ᚫ");
  t = t.replaceAll("s", "ᛋ");
  t = t.replaceAll("d", "ᛞ");
  t = t.replaceAll("f", "ᚠ");
  t = t.replaceAll("g", "ᚷ");
  t = t.replaceAll("h", "ᚻ");
  t = t.replaceAll("j", "ᚷᚻ");
  t = t.replaceAll("k", "ᚳ");
  t = t.replaceAll("l", "ᛚ");
  t = t.replaceAll("z", "ᛋ");
  t = t.replaceAll("x", "ᛉ");
  t = t.replaceAll("c", "ᚳ");
  t = t.replaceAll("v", "ᚠ");
  t = t.replaceAll("b", "ᛒ");
  t = t.replaceAll("n", "ᚾ");
  t = t.replaceAll("m", "ᛗ");

  // other
  t = t.replaceAll("&", "⁊");

  if (document.settings.space.checked) {
    t = t.replaceAll(" ", "᛫");
  }
  for (const punct of PUNCT) {
    t = t.replaceAll(punct + "᛫", punct + " ");
  }
  t = t.replaceAll(".᛫", ". ");
  t = t.replaceAll("᛫(", " (");

  return t;
}

function isPronunciationConversionEnabled() {
  const settingsForm = document.settings;
  if (!settingsForm || typeof settingsForm !== "object") {
    return true;
  }
  const checkbox = settingsForm.pronunciation;
  if (typeof checkbox === "undefined" || checkbox === null) {
    return true;
  }
  return !!checkbox.checked;
}

function convertString(input) {
  const source = Array.from((input ?? "").toString().toLowerCase());
  let working = "";

  for (const ch of source) {
    working += ch;
    working = applyConversionPass(working);
  }

  let stabilized = applyConversionPass(working);
  let iterations = 0;
  while (stabilized !== working && iterations < 4) {
    working = stabilized;
    stabilized = applyConversionPass(working);
    iterations += 1;
  }

  return working;
}

function convertWordFallback(latin, boundaryChar = " ") {
  if (!latin) {
    return "";
  }
  const boundary = boundaryChar ?? "";
  const baseInput = boundary ? latin + boundary : latin;
  let converted = convertString(baseInput);
  converted = convertString(converted);
  if (!boundary) {
    return converted;
  }
  const boundaryConverted = convertString(boundary);
  if (boundaryConverted && converted.endsWith(boundaryConverted)) {
    return converted.slice(0, converted.length - boundaryConverted.length);
  }
  if (converted.endsWith(boundary)) {
    return converted.slice(0, converted.length - boundary.length);
  }
  return converted;
}

function convertWord(latin, boundaryChar = " ") {
  if (!latin) {
    return "";
  }

  const boundary = boundaryChar ?? "";
  const baseInput = boundary ? latin + boundary : latin;
  const e2r = window.e2r;
  console.log(e2r, e2r.translate_js);

  if (e2r && e2r.translate_js) {
    try {
      let wasmResult = e2r.translate_js(baseInput, boundary);
      if (
        typeof wasmResult === "string" &&
        wasmResult !== baseInput &&
        wasmResult !== latin
      ) {
        console.log("WASM conversion succeeded", { latin, boundary, wasmResult });
        let converted = wasmResult;
        if (boundary) {
          const boundaryConverted = convertString(boundary);
          if (boundaryConverted && converted.endsWith(boundaryConverted)) {
            converted = converted.slice(0, converted.length - boundaryConverted.length);
          } else if (converted.endsWith(boundary)) {
            converted = converted.slice(0, converted.length - boundary.length);
          }
        }
        if (converted === latin) {
          // fallback to JS if no change
          return convertWordFallback(latin, boundary);
        }
        return converted;
      }
    } catch (error) {
      // fall back to JS pipeline below if WASM call fails
    }
  }

  return convertWordFallback(latin, boundary);
}

function applyCompletedWordConversions(textValue, completions, rawValue) {
  if (!Array.isArray(completions) || completions.length === 0) {
    return textValue;
  }

  let result = textValue;
  for (let i = completions.length - 1; i >= 0; i -= 1) {
    const completion = completions[i];
    if (!completion || !completion.word) {
      continue;
    }

    const { boundary, start, end } = completion;
    if (typeof start !== "number" || typeof end !== "number" || start > end) {
      continue;
    }

    const boundaryChar = boundary ?? " ";
    const desiredRunes = convertWord(completion.word, boundaryChar);
    const prefixRunes = convertString(rawValue.slice(0, start));

    const sliceWithBoundary = rawValue.slice(start, end) + boundaryChar;
    const convertedWithBoundary = convertString(sliceWithBoundary);
    const boundaryConverted = convertString(boundaryChar);

    let wordRunesLength = convertedWithBoundary.length;
    if (boundaryConverted && convertedWithBoundary.endsWith(boundaryConverted)) {
      wordRunesLength -= boundaryConverted.length;
    }

    const replaceStart = prefixRunes.length;
    const replaceEnd = replaceStart + wordRunesLength;

    result = result.slice(0, replaceStart) + desiredRunes + result.slice(replaceEnd);
  }

  return result;
}

function updateText() {
  const rawValue = text.value;
  const prevValue = lastRunifiedValue;
  const rawLen = rawValue.length;
  const selectionStart = typeof text.selectionStart === "number" ? text.selectionStart : rawLen;
  const selectionEnd = typeof text.selectionEnd === "number" ? text.selectionEnd : selectionStart;
  const trackingEnabled = selectionStart === selectionEnd && selectionEnd === rawLen;
  const pronunciationEnabled = isPronunciationConversionEnabled();
  const trackingAllowed = trackingEnabled && pronunciationEnabled;

  let completedWords = [];
  if (!trackingAllowed) {
    currentLatinWord = "";
    currentWordStartIndex = null;
  } else if (prevValue !== rawValue) {
    const { removed, added, insertionIndex } = diffStrings(prevValue, rawValue);
    if (removed.length > 0) {
      handleRemoval(removed);
    }
    if (added.length > 0) {
      completedWords = handleAddition(added.toLowerCase(), insertionIndex, rawValue);
    }
  }

  const len = rawLen;
  let pos = selectionStart;
  let t = convertString(rawValue);

  if (trackingAllowed && completedWords.length > 0) {
    t = applyCompletedWordConversions(t, completedWords, rawValue);
  }

  text.value = t;
  pos += t.length - len;
  text.selectionStart = pos;
  text.selectionEnd = pos;
  lastRunifiedValue = t;
}
text.oninput = updateText;

function copyText() {
  // Treat as if a word boundary was reached: convert any unfinished word
  if (currentLatinWord && typeof currentWordStartIndex === "number") {
    // Find the boundary char (simulate as space)
    const boundaryChar = " ";
    // Convert the unfinished word
    const desiredRunes = convertWord(currentLatinWord, boundaryChar);
    // Get prefix and suffix in runes
    const rawValue = text.value;
    const prefixRunes = convertString(rawValue.slice(0, currentWordStartIndex));
    const sliceWithBoundary = rawValue.slice(currentWordStartIndex) + boundaryChar;
    const convertedWithBoundary = convertString(sliceWithBoundary);
    const boundaryConverted = convertString(boundaryChar);

    let wordRunesLength = convertedWithBoundary.length;
    if (boundaryConverted && convertedWithBoundary.endsWith(boundaryConverted)) {
      wordRunesLength -= boundaryConverted.length;
    }

    const replaceStart = prefixRunes.length;
    const replaceEnd = replaceStart + wordRunesLength;

    // Replace the unfinished word in the textarea value
    text.value = text.value.slice(0, replaceStart) + desiredRunes + text.value.slice(replaceEnd);

    // Reset tracking state
    currentLatinWord = "";
    currentWordStartIndex = null;
    lastRunifiedValue = text.value;
  }

  text.select();
  text.setSelectionRange(0, 99999);
  document.execCommand("copy");
  //alert("Copied text to clipboard\nᚳᛟᛈᛁᛁᛞ᛫ᛏᛖᛉᛏ᛫ᛏᚣ᛫ᚳᛚᛁᛈᛒᚩᚱᛞ");
  text.setSelectionRange(0, 0);
}
function clearText() {
  if (confirm("Clear text?\nᚳᛚᛁᛁᚱ᛫ᛏᛖᛉᛏ?")) {
    text.value = "";
    currentLatinWord = "";
    currentWordStartIndex = null;
    lastRunifiedValue = "";
  }
}
