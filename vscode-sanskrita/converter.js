// संस्कृता roman → Devanagari converter (mirror of devanagarify() in sanskrita.py)
"use strict";

const ALIASES = {
  maanaya: "मानय", manaya: "मानय", manay: "मानय",
  dhruva: "ध्रुव", dhruv: "ध्रुव",
  yadi: "यदि",
  atha: "अथ",
  anyathaa: "अन्यथा", anyatha: "अन्यथा",
  yaavat: "यावत्", yavat: "यावत्",
  satyam: "सत्यम्",
  asatyam: "असत्यम्",
  shuunyam: "शून्यम्", shunyam: "शून्यम्",
  cha: "च", ca: "च",
  vaa: "वा",
  na: "न",
  virama: "विरम", viram: "विरम",
  anuvarta: "अनुवर्त", anuvart: "अनुवर्त",
  vidhi: "विधि",
  phalam: "फलम्",
  pratyekam: "प्रत्येकम्",
  iti: "इति",
  vargah: "वर्गः", varga: "वर्गः",
  srja: "सृज", srija: "सृज",
  ayam: "अयम्",
  prayata: "प्रयत", prayat: "प्रयत",
  doshe: "दोषे",
  aanaya: "आनय", anaya: "आनय",
  aarambha: "आरम्भ", arambha: "आरम्भ",
  kartaa: "कर्ता", karta: "कर्ता",
  karma: "कर्म",
  karana: "करण",
  sampradaana: "सम्प्रदान", sampradana: "सम्प्रदान",
  apaadaana: "अपादान", apadana: "अपादान",
  adhikarana: "अधिकरण",
  purnankah: "पूर्णाङ्कः", purnanka: "पूर्णाङ्कः",
  dashamanshah: "दशमांशः", dashamansha: "दशमांशः",
  satyasatyam: "सत्यासत्यम्",
  suchee: "सूची", suchi: "सूची",
  koshah: "कोशः", kosha: "कोशः",
  vada: "वद", vad: "वद",
  prccha: "पृच्छ", pruccha: "पृच्छ", puccha: "पृच्छ",
  vaakyam: "वाक्यम्", vakyam: "वाक्यम्",
  sankhyaa: "सङ्ख्या", sankhya: "सङ्ख्या",
  prakarah: "प्रकारः", prakara: "प्रकारः",
  dairghyam: "दैर्घ्यम्", dairghya: "दैर्घ्यम्",
  yojaya: "योजय",
  apanaya: "अपनय",
  kunjikaah: "कुञ्जिकाः", kunjikah: "कुञ्जिकाः", kunjika: "कुञ्जिकाः",
  kramaya: "क्रमय"
};

const DEV_DIGITS = "०१२३४५६७८९";

function devanagarify(src) {
  let out = "";
  let i = 0;
  const n = src.length;
  while (i < n) {
    const c = src[i];
    if (c === '"') {                       // strings stay untouched
      let j = i + 1;
      while (j < n && src[j] !== '"') {
        if (src[j] === "\\") j++;
        j++;
      }
      j = Math.min(j + 1, n);
      out += src.slice(i, j); i = j; continue;
    }
    if (c === "#") {                       // comments stay untouched
      let j = src.indexOf("\n", i);
      if (j === -1) j = n;
      out += src.slice(i, j); i = j; continue;
    }
    if (c === "|") { out += "।"; i++; continue; }
    if (/[A-Za-z_]/.test(c)) {             // ASCII word: alias -> canonical
      let j = i;
      while (j < n && /[A-Za-z0-9_]/.test(src[j])) j++;
      const word = src.slice(i, j);
      out += ALIASES[word] || word;
      i = j; continue;
    }
    if (/[0-9]/.test(c)) { out += DEV_DIGITS[+c]; i++; continue; }
    out += c; i++;
  }
  return out;
}

module.exports = { devanagarify };
