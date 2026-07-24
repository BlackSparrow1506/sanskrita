#!/bin/bash
# संस्कृता v0.3 "फलम्" update video — narration
# Run:  bash make_voice_v8.sh
set -e
cd "$(dirname "$0")"
mkdir -p v6
python3 -m pip install --user --quiet edge-tts

VOICE="hi-IN-SwaraNeural"
speak() {
    python3 -m edge_tts --voice "$VOICE" --rate="-5%" --text "$2" --write-media "v6/line$1.mp3"
    echo "✓ v6/line$1.mp3"
}

speak 1 "संस्कृता का नया version आ गया — शून्य दशमलव तीन, फलम्! और यह अब तक का सबसे बड़ा update है।"
speak 2 "सबसे बड़ी बात — अब आप अपनी संस्कृता files को import कर सकते हैं। यानी संस्कृत में अपनी libraries बनाइए, और बड़े projects लिखिए!"
speak 3 "नया वाक्यकर्म module — text को तोड़िए, जोड़िए, खोजिए, बदलिए — सब संस्कृत में। और परिधिः से गिनती अब सिर्फ़ एक line में।"
speak 4 "Real-world programs भी तैयार हैं — घर का बजट, loan की EMI, text का विश्लेषण, कक्षा की अङ्कतालिका — सब repo में, चलाने के लिए ready।"
speak 5 "और अंदर से भी मज़बूती — हर बदलाव पर अपने आप tests चलते हैं, benchmarks publish होते हैं, और Unicode safety अब पक्की है।"
speak 6 "v शून्य दशमलव तीन — GitHub पर अभी live है। Link नीचे। जयतु संस्कृतम्!"

echo ""
echo "सिद्धम् ✓ v0.3 update narration ready (video/v6/). Tell Claude!"
