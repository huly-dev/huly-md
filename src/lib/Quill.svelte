<script lang="ts">
  import { onMount } from 'svelte'

  import Quill, { type EmitterSource } from 'quill'
  import Delta, { Op } from 'quill-delta'

  import 'quill/dist/quill.core.css'

  import { addListener, applyDelta, getTextValue } from './peer_rs'

  export let docId: string
  export let path: string
  export let origin: string

  let editor: HTMLElement

  onMount(() => {
    const quill = new Quill(editor)
    quill.setContents([])
    quill.on('text-change', (delta: Delta, _, source: EmitterSource) => {
      if (source !== 'api') applyDelta(docId, path, origin, delta.ops)
    })
    addListener(docId, path, (orgn: string, ops: Op[]) => {
      if (orgn !== origin) quill.updateContents(ops, 'api')
    })
  })
</script>

<div style="height: 240px; width: 100%; border: #fff solid 1px">
  <div bind:this={editor} />
</div>
