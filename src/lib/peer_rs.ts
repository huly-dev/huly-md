import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { Op } from 'quill-delta'

type OpsListener = (origin: string, ops: Op[]) => void

const listeners = new Map<string, OpsListener[]>()

const createCID = (docId: string, path: string) => `${docId}/${path}`

export const addListener = (docId: string, path: string, listener: OpsListener) => {
  const cid = createCID(docId, path)
  const list = listeners.get(cid)
  if (list) {
    list.push(listener)
  } else {
    listeners.set(cid, [listener])
  }
}

const removeListener = (docId: string, path: string, listener: OpsListener) => {
  const cid = createCID(docId, path)
  const list = listeners.get(cid)
  if (list) {
    const index = list.indexOf(listener)
    if (index !== -1) {
      list.splice(index, 1)
    }
  }
}

listen('doc-diff', (message) => {
  console.log('doc-diff', message)
  const { origin, docId, diff } = message.payload as HulyDocDiff
  diff.forEach((d) => {
    const cid = createCID(docId, d.id)
    const list = listeners.get(cid)
    console.log('broadcasting', cid)
    if (list) {
      list.forEach((listener) => listener(origin, d.diff))
    }
  })
})

type DocId = string

interface HulyPeer {
  readonly peerId: bigint
  getDoc(docId: DocId): HulyDoc
}

export interface HulyDoc {
  readonly docId: DocId
  getText(path: string): HulyText
  import(delta: Uint8Array): void
}

interface HulyDocDiff {
  readonly docId: DocId
  readonly origin: string
  readonly diff: HulyTextDiff[]
}

interface HulyTextDiff {
  id: string
  type: string
  diff: Op[]
}

interface HulyText {
  getContents(): Promise<Op[]>
  update(origin: string, ops: Op[]): Promise<void>
  subscribe(listener: OpsListener): void
}

// Tauri Commands

export const getTextValue = (docId: DocId, path: string): Promise<Op[]> =>
  invoke('get_text_value', { docId, path })

export const applyDelta = (
  docId: DocId,
  path: string,
  origin: string,
  delta: Op[],
): Promise<void> => invoke('apply_delta', { docId, path, origin, delta })

const after = { expand: 'after' as 'after' }

function createDoc(peerId: bigint, docId: DocId): HulyDoc {
  return {
    docId,
    import(delta: Uint8Array) {},
    getText(cid: string): HulyText {
      return {
        getContents: () => getTextValue(docId, cid),
        update: (origin, ops: Op[]) => applyDelta(docId, cid, origin, ops),
        subscribe: (listener: (origin: string, ops: Op[]) => void) =>
          addListener(docId, cid, listener),
      }
    },
  }
}

export const createPeer = (peerId: bigint): HulyPeer => ({
  peerId,
  getDoc: (docId: string) => createDoc(peerId, docId),
})
