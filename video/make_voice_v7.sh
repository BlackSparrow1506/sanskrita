#!/bin/bash
# संस्कृता Shorts v2 — informative versions with real code & output
# Run:  bash make_voice_v7.sh
set -e
cd "$(dirname "$0")"
mkdir -p v5
python3 -m pip install --user --quiet edge-tts

VOICE="hi-IN-SwaraNeural"
speak() {
    python3 -m edge_tts --voice "$VOICE" --rate="-4%" --text "$2" --write-media "v5/$1.mp3"
    echo "✓ v5/$1.mp3"
}

# --- Gītā meter short (informative) ---
speak g1 "क्या कोई प्रोग्रामिंग भाषा भगवद्गीता का छन्द पहचान सकती है? संस्कृता कर सकती है — और मैं आपको पूरा code दिखाती हूँ।"
speak g2 "पहले समझिए — छन्द यानी वैदिक meter। अनुष्टुभ्, गीता का छन्द — चार पाद, हर पाद में आठ अक्षर। कुल बत्तीस।"
speak g3 "यह रहा पूरा code — संस्कृतम् library आनय कीजिए, श्लोक दीजिए, और पूछिए — सं डॉट छन्दः।"
speak g4 "और यह रहा असली output — अनुष्टुभ्, अक्षराणि बत्तीस। Engine ने अक्षर भी गिनकर दिखाए — ध, र्म, क्षे, त्रे — संयुक्ताक्षर भी बिल्कुल सही!"
speak g5 "एक और कमाल — गायत्री मन्त्र लिखने में तेईस अक्षर का दिखता है, पर उच्चारण में चौबीस का होता है — engine दोनों पकड़ लेता है!"
speak g6 "पूरा example — छन्दःपरीक्षा डॉट सं — GitHub पर मुफ़्त है। ख़ुद चलाकर देखिए! जयतु संस्कृतम्!"

# --- Kāraka short (informative) ---
speak k1 "हर प्रोग्रामिंग भाषा में arguments का order fix होता है — पहले message, फिर receiver। उलट दिया? Bug!"
speak k2 "पर संस्कृत में क्रम मायने नहीं रखता। रामः फलं खादति — या — फलं खादति रामः। अर्थ वही! क्योंकि शब्दों के कारक ख़ुद भूमिका बताते हैं।"
speak k3 "पाणिनि के छह कारक — कर्ता, कर्म, करण, सम्प्रदान, अपादान, अधिकरण। संस्कृता इन्हें सीधे code में ले आई।"
speak k4 "यह रहा पूरा code — विधि में हर parameter के आगे उसका कारक लिखिए। कर्म — यानी क्या। सम्प्रदान — किसको। करण — किसके द्वारा।"
speak k5 "अब call कीजिए — किसी भी order में! तीन अलग orders — तीनों बार बिल्कुल वही जवाब। और ग़लत कारक दीजिए, तो engine तुरंत पकड़ लेता है।"
speak k6 "पूरा example — कारकक्रमः डॉट सं — GitHub पर। दुनिया की किसी और भाषा में यह feature नहीं है। जयतु संस्कृतम्!"

echo ""
echo "सिद्धम् ✓ Informative narrations ready (video/v5/). Tell Claude!"
