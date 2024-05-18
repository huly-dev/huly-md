<script lang="ts">
  import { onMount } from 'svelte'

  import Quill, { type EmitterSource } from 'quill'
  import Delta from 'quill-delta'
  import 'quill/dist/quill.core.css'

  import type { HulyDoc, Origin, HulyDelta } from './peer_rs'

  export let origin: Origin
  export let doc: HulyDoc
  export let path: string

  let editor: HTMLElement

  onMount(() => {
    const text = doc.getText(path)
    text.subscribe((delta: HulyDelta) => {
      quill.updateContents(delta.ops, 'api')
    })

    const quill = new Quill(editor)
    quill.setContents(text.getContents())
    quill.on('text-change', (delta: Delta, _, source: EmitterSource) => {
      if (source !== 'api') text.update(delta.ops)
    })
  })
</script>

<div style="height: 240px; width: 100%; border: #fff solid 1px">
  <div bind:this={editor} />
</div>
