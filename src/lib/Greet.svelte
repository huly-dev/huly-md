<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'

  const docId = 'my-doc'
  const cid = 'cid:root-description:Text'

  let text = ''

  function openDoc() {
    invoke('open_doc', { docId })
      .then(() => {
        text = 'open OK'
      })
      .catch((err) => {
        text = err
      })
  }

  function getTextContent() {
    invoke('get_text_content', { docId, cid })
      .then((content) => {
        text = 'OK: ' + String(content)
      })
      .catch((err) => {
        text = err
      })
  }

  async function greet() {
    text = await invoke('greet', { name: 'Andrey' })
  }
</script>

<div>Text: {text}</div>

<div>
  <button on:click={openDoc}>Open Doc</button>
  <button on:click={getTextContent}>Get Content</button>
  <button on:click={greet}>Greet</button>
</div>
