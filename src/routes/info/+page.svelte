<script lang="ts">
  // Static info page — no logic needed
</script>

<div class="app-layout">
  <header class="app-header">
    <div class="header-inner">
      <div>
        <h1 class="app-title">Regex Quick Guide</h1>
        <p class="app-subtitle">Common patterns for the Find &amp; Replace operation.</p>
      </div>
      <a href="/" class="back-link">← Back to Renamer</a>
    </div>
  </header>

  <main class="content">

    <!-- ── Basics ─────────────────────────────────────────────────────── -->
    <section class="panel">
      <h2 class="panel-title">Basics</h2>
      <table class="guide-table">
        <thead>
          <tr><th>Pattern</th><th>Matches</th><th>Example</th></tr>
        </thead>
        <tbody>
          <tr><td><code>.</code></td><td>Any single character (except newline)</td><td><code>a.c</code> → "abc", "a1c"</td></tr>
          <tr><td><code>\d</code></td><td>Any digit (0–9)</td><td><code>\d\d</code> → "42"</td></tr>
          <tr><td><code>\w</code></td><td>Word character (letter, digit, _)</td><td><code>\w+</code> → "hello_1"</td></tr>
          <tr><td><code>\s</code></td><td>Whitespace (space, tab)</td><td><code>a\sb</code> → "a b"</td></tr>
          <tr><td><code>\D</code></td><td>Non-digit</td><td><code>\D+</code> → "abc"</td></tr>
          <tr><td><code>\W</code></td><td>Non-word character</td><td><code>\W</code> → "-", " "</td></tr>
        </tbody>
      </table>
    </section>

    <!-- ── Anchors & Boundaries ───────────────────────────────────────── -->
    <section class="panel">
      <h2 class="panel-title">Anchors &amp; Boundaries</h2>
      <table class="guide-table">
        <thead>
          <tr><th>Pattern</th><th>Matches</th><th>Example</th></tr>
        </thead>
        <tbody>
          <tr><td><code>^</code></td><td>Start of string</td><td><code>^IMG</code> → "IMG_001" but not "my_IMG"</td></tr>
          <tr><td><code>$</code></td><td>End of string</td><td><code>_v2$</code> → "photo_v2" but not "photo_v2_edit"</td></tr>
          <tr><td><code>\b</code></td><td>Word boundary</td><td><code>\bcat\b</code> → "cat" but not "catch"</td></tr>
        </tbody>
      </table>
    </section>

    <!-- ── Quantifiers ────────────────────────────────────────────────── -->
    <section class="panel">
      <h2 class="panel-title">Quantifiers</h2>
      <table class="guide-table">
        <thead>
          <tr><th>Pattern</th><th>Meaning</th><th>Example</th></tr>
        </thead>
        <tbody>
          <tr><td><code>*</code></td><td>0 or more</td><td><code>ab*c</code> → "ac", "abc", "abbc"</td></tr>
          <tr><td><code>+</code></td><td>1 or more</td><td><code>\d+</code> → "1", "42", "007"</td></tr>
          <tr><td><code>?</code></td><td>0 or 1 (optional)</td><td><code>colou?r</code> → "color", "colour"</td></tr>
          <tr><td><code>{"{n}"}</code></td><td>Exactly n times</td><td><code>\d{"{4}"}</code> → "2024"</td></tr>
          <tr><td><code>{"{n,m}"}</code></td><td>Between n and m times</td><td><code>\d{"{2,4}"}</code> → "24", "2024"</td></tr>
        </tbody>
      </table>
    </section>

    <!-- ── Groups & Capture ───────────────────────────────────────────── -->
    <section class="panel">
      <h2 class="panel-title">Groups &amp; Capture</h2>
      <table class="guide-table">
        <thead>
          <tr><th>Pattern</th><th>Meaning</th><th>Example</th></tr>
        </thead>
        <tbody>
          <tr><td><code>(abc)</code></td><td>Capture group — reuse with <code>$1</code>, <code>$2</code>…</td><td><code>(\d+)</code> captures the number</td></tr>
          <tr><td><code>(?:abc)</code></td><td>Non-capturing group</td><td><code>(?:IMG|DSC)_\d+</code></td></tr>
          <tr><td><code>[abc]</code></td><td>Character class — any of a, b, c</td><td><code>[aeiou]</code> → any vowel</td></tr>
          <tr><td><code>[^abc]</code></td><td>Negated class — anything except a, b, c</td><td><code>[^0-9]</code> → non-digit</td></tr>
          <tr><td><code>a|b</code></td><td>Alternation — a or b</td><td><code>jpg|png|gif</code></td></tr>
        </tbody>
      </table>
    </section>

    <!-- ── Practical Examples ─────────────────────────────────────────── -->
    <section class="panel">
      <h2 class="panel-title">Practical File-Renaming Examples</h2>
      <table class="guide-table examples-table">
        <thead>
          <tr><th>Goal</th><th>Find</th><th>Replace with</th></tr>
        </thead>
        <tbody>
          <tr>
            <td>Remove leading numbers</td>
            <td><code>^\d+[-_\s]?</code></td>
            <td><em>(empty)</em></td>
          </tr>
          <tr>
            <td>Strip spaces</td>
            <td><code>\s+</code></td>
            <td><code>_</code></td>
          </tr>
          <tr>
            <td>Remove duplicate underscores</td>
            <td><code>_{"{2,}"}</code></td>
            <td><code>_</code></td>
          </tr>
          <tr>
            <td>Swap "IMG_NNNN" → "photo_NNNN"</td>
            <td><code>^IMG_(\d+)</code></td>
            <td><code>photo_$1</code></td>
          </tr>
          <tr>
            <td>Remove trailing " copy"</td>
            <td><code>\s*copy$</code></td>
            <td><em>(empty)</em></td>
          </tr>
          <tr>
            <td>Normalise date separators (2024.01.05 → 2024-01-05)</td>
            <td><code>(\d{"{4}"})[./](\d{"{2}"})[./](\d{"{2}"})</code></td>
            <td><code>$1-$2-$3</code></td>
          </tr>
        </tbody>
      </table>
    </section>

    <!-- ── Tips ───────────────────────────────────────────────────────── -->
    <section class="panel tips-panel">
      <h2 class="panel-title">Tips</h2>
      <ul class="tips-list">
        <li>Escape special characters with <code>\</code> when you mean them literally — e.g. <code>\.</code> matches a real dot, not any character.</li>
        <li>Use capture groups <code>(...)</code> and back-references <code>$1</code>, <code>$2</code> in the Replace field to rearrange parts of a filename.</li>
        <li>The preview table updates live — experiment freely before executing.</li>
        <li>Patterns are case-sensitive by default. Use <code>[Aa]</code> or alternation to match both cases.</li>
      </ul>
    </section>

  </main>
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    background-color: #f3f4f6;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }

  .app-header {
    padding: 1rem 1.5rem 0.75rem;
    background-color: #fff;
    border-bottom: 1px solid #e5e7eb;
  }

  .header-inner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .app-title {
    font-size: 1.25rem;
    font-weight: 700;
    color: #111827;
    margin: 0 0 0.125rem;
  }

  .app-subtitle {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  .back-link {
    font-size: 0.875rem;
    font-weight: 500;
    color: #4f46e5;
    text-decoration: none;
    white-space: nowrap;
  }

  .back-link:hover {
    text-decoration: underline;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem 1.5rem;
    max-width: 900px;
    width: 100%;
    margin: 0 auto;
    box-sizing: border-box;
  }

  .panel {
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    padding: 1rem;
  }

  .panel-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: #374151;
    margin: 0 0 0.75rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid #f3f4f6;
  }

  .guide-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8125rem;
  }

  .guide-table th {
    text-align: left;
    font-weight: 600;
    color: #6b7280;
    padding: 0.375rem 0.75rem;
    border-bottom: 1px solid #e5e7eb;
    white-space: nowrap;
  }

  .guide-table td {
    padding: 0.5rem 0.75rem;
    color: #374151;
    border-bottom: 1px solid #f3f4f6;
    vertical-align: top;
  }

  .guide-table tr:last-child td {
    border-bottom: none;
  }

  .guide-table code {
    font-family: 'Fira Code', 'Cascadia Code', Menlo, monospace;
    font-size: 0.8125rem;
    background-color: #f3f4f6;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: #4f46e5;
  }

  .examples-table td:first-child {
    color: #6b7280;
    font-style: italic;
    white-space: nowrap;
  }

  .tips-panel .panel-title {
    margin-bottom: 0.5rem;
  }

  .tips-list {
    margin: 0;
    padding-left: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .tips-list li {
    font-size: 0.8125rem;
    color: #374151;
    line-height: 1.5;
  }

  .tips-list code {
    font-family: 'Fira Code', 'Cascadia Code', Menlo, monospace;
    font-size: 0.8125rem;
    background-color: #f3f4f6;
    padding: 0.125rem 0.375rem;
    border-radius: 0.25rem;
    color: #4f46e5;
  }

  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    .app-layout { background-color: #111827; }
    .app-header { background-color: #1f2937; border-bottom-color: #374151; }
    .app-title { color: #f9fafb; }
    .app-subtitle { color: #9ca3af; }
    .back-link { color: #818cf8; }
    .panel { background-color: #1f2937; border-color: #374151; }
    .panel-title { color: #d1d5db; border-bottom-color: #374151; }
    .guide-table th { color: #9ca3af; border-bottom-color: #374151; }
    .guide-table td { color: #d1d5db; border-bottom-color: #374151; }
    .guide-table code { background-color: #111827; color: #a5b4fc; }
    .tips-list li { color: #d1d5db; }
    .tips-list code { background-color: #111827; color: #a5b4fc; }
    .examples-table td:first-child { color: #9ca3af; }
  }
</style>
