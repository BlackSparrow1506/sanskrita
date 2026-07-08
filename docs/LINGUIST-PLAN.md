# GitHub Linguist Registration Plan (future milestone)

Goal: `.सं` files show as **Sanskrita** (color: saffron) on all of GitHub.

## Eligibility gate (their rule, verified July 2026)

~2,000 `.सं` files indexed on public GitHub in the last year, spread across many
unique users/repos (they exclude the creator's own repos). Check readiness with:
`https://github.com/search?type=code&q=NOT+is%3Afork+path%3A*.%E0%A4%B8%E0%A4%82`

## When eligible — the PR contents (fork: BlackSparrow1506/linguist)

1. Entry in `lib/linguist/languages.yml` (alphabetical position):

```yaml
Sanskrita:
  type: programming
  color: "#FF9933"
  extensions:
  - ".सं"
  - ".sam"
  tm_scope: source.sanskrita
  ace_mode: text
  language_id: (generate with script/update-ids)
```

Note: `.sam` may conflict with other languages by then — if so, a heuristic
disambiguating on danda `।` / Devanagari keywords will be needed.

2. Grammar: `script/add-grammar https://github.com/BlackSparrow1506/sanskrita`
   (uses vscode-sanskrita/syntaxes/sanskrita.tmLanguage.json — MIT licensed ✓)

3. Samples: real-world code, NOT hello-world — use वर्गाः.सं, संस्कृतम्.सं,
   प्रतिमानानि.सं (MIT, we own them; state this in the PR).

4. Fill their PR template completely + include the search link showing usage.

## How we get to 2,000 files (the real work)

Every public repo by students/scholars counts. Growth channels:
Sanskrit universities & pāṭhaśālās, Samskrita Bharati, r/sanskrit,
Sanskrit-computational-linguistics community, coding-in-schools programs,
tutorials that end with "publish your exercises on GitHub."
