<script lang="ts">
  import Quill from './lib/Quill.svelte'
  import { createPeer } from './lib/peer'

  const peer1 = createPeer(1n)
  const peer2 = createPeer(2n)

  const doc1 = peer1.getDoc('x')
  const doc2 = peer2.getDoc('x')

  function editor1(event: CustomEvent) {
    console.log('editor1', event.detail)
  }

  function editor2(event: CustomEvent) {
    console.log('editor2', event.detail)
  }

  function editor3(event: CustomEvent) {
    console.log('editor3', event.detail)
  }
</script>

<main class="container">
  <h1>Welcome to Tauri!</h1>

  <p>Click on the Tauri, Vite, and Svelte logos to learn more.</p>

  <div class="row">
    <Quill origin="e1" doc={doc1} path={'description'} on:change={editor1} />
  </div>

  <div class="row">
    <Quill origin="e2" doc={doc1} path={'description'} on:change={editor2} />
  </div>

  <div class="row">
    <Quill origin="e3" doc={doc2} path={'description'} on:change={editor3} />
  </div>
</main>

<style>
</style>
