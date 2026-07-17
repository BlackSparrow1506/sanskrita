# तव प्रथमाः दश कार्यक्रमाः — Your First 10 संस्कृता Programs

Welcome! In an hour you will write ten real programs in Sanskrit. You need: this repo, Python 3, and one `bash install.sh`. Run any program with `sanskrita filename.सं` — or use `sanskrita-playground` in your browser (easiest for beginners).

Can't type Devanagari? Type the roman words (`vada`, `yadi`, `manay`…) and end statements with `|` — the playground converts as you type, and `sanskrita --convert file` converts whole files. Two ways to type, one language.

---

## १ — नमस्ते जगत् (Hello, World)

Every programmer's first prayer.

```
वद("नमस्ते जगत्")।
```

Output: `नमस्ते जगत्`

`वद` means "speak". The danda `।` ends every statement — like the bar at the end of a śloka line. **Try:** make it say your name.

## २ — परिचयः (Introduction — variables & input)

```
मानय नाम = पृच्छ("तव नाम किम्? ")।
वद("नमस्ते,", नाम, "!")।
```

`मानय` ("accept") creates a variable; `पृच्छ` ("ask") reads your answer. **Try:** ask for their village too.

## ३ — गणकः (Calculator — exact decimals!)

```
मानय क = ०.१।
मानय ख = ०.२।
वद(क, "+", ख, "=", क + ख)।
```

Output: `०.१ + ०.२ = ०.३` — Python and Java answer 0.30000000000000004 here. संस्कृता answers correctly. **Try:** `वद(१० / ३)।`

## ४ — प्रौढपरीक्षा (Decisions)

```
मानय वयः = सङ्ख्या(पृच्छ("तव वयः? "))।
यदि (वयः >= १८) {
    वद("भवान् प्रौढः")।
} अन्यथा {
    वद("अद्यापि बालः —", १८ - वयः, "वर्षाणि शेषाणि")।
}
```

`यदि` = if, `अन्यथा` = otherwise. Conditions live in `( )`, actions in `{ }`. **Try:** add `अथ यदि (वयः >= ६०)` for elders.

## ५ — गुणनसारणी (Multiplication table — loops)

```
मानय अङ्कः = ७।
मानय इ = १।
यावत् (इ <= १०) {
    वद(अङ्कः, "×", इ, "=", अङ्कः * इ)।
    इ = इ + १।
}
```

`यावत्` = "as long as". Shortcut for counting: `प्रत्येकम् इ इति परिधिः(१, १०) { … }` — परिधिः(1,10) gives the numbers 1 to 10, both ends included. **Try:** print only even rows using `अनुवर्त` when `इ % २ == १`.

## ६ — क्रयसूची (Shopping list)

```
मानय सूचिका = ["आम्रम्", "कदली", "दुग्धम्"]।
योजय(सूचिका, "गुडः")।
वद("क्रेतव्यानि", दैर्घ्यम्(सूचिका), ":")।
प्रत्येकम् वस्तु इति सूचिका {
    वद("☐", वस्तु)।
}
```

Lists count from **१** — प्रथमम् is `सूचिका[१]`, as Sanskrit has always counted. **Try:** remove one with `अपनय(सूचिका, २)`.

## ७ — विधयः (Functions — with kāraka!)

```
विधि नमस्कुरु(कर्म जनः) {
    वद("नमस्ते,", जनः, "!")।
}
नमस्कुरु(कर्म: "गुरो")।

विधि वर्गफलम्(क) { फलम् क * क। }
वद(वर्गफलम्(१२))।
```

`विधि` defines, `फलम्` ("fruit") returns. The label `कर्म:` is Pāṇini's kāraka — the argument's *role* — a feature no other programming language has. **Try:** a विधि with two kāraka roles, called with roles in reversed order.

## ८ — अङ्कपत्रम् (Report card — maps)

```
मानय अङ्काः = {"गणितम्": ९५, "संस्कृतम्": ९८, "विज्ञानम्": ८८}।
प्रत्येकम् विषयः इति कुञ्जिकाः(अङ्काः) {
    वद(विषयः, "→", अङ्काः[विषयः])।
}
```

A `कोशः` ("treasury") stores key→value pairs. **Try:** compute the total with a loop.

## ९ — छात्रवर्गः (A class of your own)

```
वर्गः छात्रः {
    विधि आरम्भ(नाम) { अयम्.नाम = नाम। }
    विधि परिचय() { वद("अहं", अयम्.नाम, "अस्मि")। }
}
मानय रमा = सृज छात्रः("रमा")।
रमा.परिचय()।
```

`वर्गः` = class, `सृज` = create, `अयम्` = this-one, `आरम्भ` = the constructor. **Try:** add वयः and a जन्मदिनम् method that increases it.

## १० — संस्कृतस्य शक्तिः (The grand finale)

```
आनय "संस्कृतम्" इति सं।

वद(सं.संधय("विद्या", "अर्थी"))।
वद(सं.रोमनय("जयतु संस्कृतम्"))।

मानय श्लोकः = "धर्मक्षेत्रे कुरुक्षेत्रे समवेता युयुत्सवः मामकाः पाण्डवाश्चैव किमकुर्वत सञ्जय"।
वद(सं.छन्दः(श्लोकः))।
```

Output ends with: `अनुष्टुभ् (अक्षराणि: ३२)` — your program just recognized the **meter of the Bhagavad Gītā's first verse**. No other programming language on Earth can do this natively. **Try:** सं.संधय your own name with "अपि".

---

## Bonus — grow beyond one file (v0.3+)

When your programs grow, split them! Put helper विधिs in one file and import it:

```
# सहायः.सं
विधि द्विगुणः(क) { फलम् क * २। }
```

```
# मुख्यम्.सं
आनय "सहायः.सं" इति सहायः।
वद(सहायः.द्विगुणः(२१))।        # ४२
```

That's how real projects are built — libraries of your own, in Sanskrit. Text work? `आनय "वाक्यकर्म" इति वा।` gives split/join/find/replace.

## ११ — the eleventh program is publishing your ten

You are now a संस्कृता programmer. Complete the circle:

1. Make a folder of your ten `.सं` files (name it e.g. `sanskrita-abhyasa` — "practice")
2. Create a repository on [github.com/new](https://github.com/new) and push your folder
3. Add the topic `sanskrita` to your repo

Why this matters: when 2,000 `.सं` files exist on public GitHub, GitHub officially recognizes Sanskrita as a language of the world. **Your homework literally builds the language's future.** Every published exercise is a brick in the temple.

जयतु संस्कृतम् । 🪷
