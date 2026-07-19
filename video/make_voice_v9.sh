#!/bin/bash
# "Why Sanskrit was engineered — and why संस्कृता stands apart" — narration
# Run:  bash make_voice_v9.sh
set -e
cd "$(dirname "$0")"
mkdir -p v7
python3 -m pip install --user --quiet edge-tts

VOICE="hi-IN-SwaraNeural"
speak() {
    python3 -m edge_tts --voice "$VOICE" --rate="-6%" --text "$2" --write-media "v7/line$1.mp3"
    echo "✓ v7/line$1.mp3"
}

speak 1 "संस्कृत — नाम का ही अर्थ है: संस्कारित, refined। यह भाषा बस evolve नहीं हुई — जान-बूझकर engineer की गई थी। मक़सद: ज्ञान को बिना किसी error के, हज़ारों साल तक transmit करना।"
speak 2 "Proof चाहिए? वेद हज़ारों साल तक बिना लिखे, मुँह-ज़ुबानी transmit हुए — घनपाठ जैसी recitation techniques के साथ, जो असल में error-correction के algorithms थीं। शब्दों को आगे-पीछे, उलट-पलट कर दोहराया जाता — ताकि एक अक्षर भी न बदले।"
speak 3 "वर्णमाला भी engineering है — क, च, ट, त, प — आवाज़ें मुँह में जहाँ से निकलती हैं, ठीक उसी क्रम में — गले से होंठों तक। दुनिया की पहली scientific phonetic classification।"
speak 4 "पिङ्गल ने छन्दशास्त्र में लघु-गुरु के combinations गिने — यह binary counting थी, आज के computers से दो हज़ार साल पहले। और वही परम्परा Fibonacci numbers तक पहुँची — Fibonacci के जन्म से सदियों पहले, मात्रामेरु के नाम से।"
speak 5 "सत्रह सौ छियासी में William Jones ने Calcutta में कहा — संस्कृत, Greek से ज़्यादा perfect, Latin से ज़्यादा समृद्ध। इसी observation से modern linguistics का जन्म हुआ।"
speak 6 "अब संस्कृता की बात। दुनिया की हर programming language English के शब्दों पर बनी है। संस्कृता पहली serious भाषा है जो उस भाषा पर खड़ी है — जो ख़ुद engineer की गई थी। Foundation ही अलग है।"
speak 7 "इसीलिए इसमें वो है जो किसी और में नहीं — पाणिनि के कारक, function arguments में। छन्द और सन्धि, भाषा के अंदर built-in। दशमलव हमेशा शुद्ध। और errors — संस्कृत और English, दोनों में।"
speak 8 "और पूरा सच भी सुनिए — आज संस्कृता एक prototype है। धीमी है। छोटी है। पर open source है, हर हफ़्ते बढ़ रही है — और जो यह कर सकती है, वो दुनिया की कोई और programming language नहीं कर सकती। जयतु संस्कृतम्!"

echo ""
echo "सिद्धम् ✓ Heritage narration ready (video/v7/). Tell Claude!"
