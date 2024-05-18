<script lang="ts">
  import Quill from './lib/Quill.svelte'
  import Greet from './lib/Greet.svelte'

  const doc1 = 'x'

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

  <div class="row">
    <Greet />
  </div>

  <div class="row">
    <Quill origin="e1" docId={doc1} path={'description'} on:change={editor1} />
  </div>

  <div class="row">
    <Quill origin="e2" docId={doc1} path={'description'} on:change={editor2} />
  </div>

  <!-- <div class="row">
    <Quill origin="e3" doc={doc2} path={'description'} on:change={editor3} />
  </div> -->
</main>

<style>
</style>
