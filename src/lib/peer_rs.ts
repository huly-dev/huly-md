import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import { Loro, type LoroEventBatch } from 'loro-crdt'
import { Op } from 'quill-delta'

listen('diff', (delta) => {
  console.log('diff', delta)
})

type DocId = string
export type Origin = string

interface HulyPeer {
  readonly peerId: bigint
  getDoc(docId: DocId): HulyDoc
}

export interface HulyDoc {
  readonly docId: DocId
  getText(path: string): HulyText
  import(delta: Uint8Array): void
}

export interface HulyDelta {
  readonly docId: DocId
  readonly ops: Op[]
}

interface HulyText {
  getContents(): Op[]
  update(ops: Op[]): Promise<void>
  subscribe(listener: (delta: HulyDelta) => void): void
}

// type RemoteDelta = {
//   readonly docId: DocId
//   readonly delta: Uint8Array
// }

// Tauri Commands

const getTextValue = (docId: DocId, cid: string): Promise<object> =>
  invoke('get_text_value', { docId, cid })
const applyDelta = (docId: DocId, cid: string, origin: string, delta: Op[]): Promise<object> =>
  invoke('apply_delta', { docId, cid, origin, delta })
const subscribe = (docId: DocId, cid: string): Promise<number> =>
  invoke('subscribe', { docId, cid })

const after = { expand: 'after' as 'after' }
let originSeq: number = 0

function createDoc(peerId: bigint, docId: DocId) {
  const doc = new Loro()
  doc.setPeerId(peerId)
  doc.configTextStyle({ bold: after, italic: after, list: after, indent: after, link: after })

  return {
    docId,
    import(delta: Uint8Array) {
      doc.import(delta)
    },
    getText(path: string): HulyText {
      const origin = String(++originSeq)
      const text = doc.getText(path)
      const cid = `cid:root-${path}:Text`

      return {
        getContents() {
          return text.toDelta()
        },
        async update(ops: Op[]) {
          // const current = doc.version()
          console.log('update', ops)
          text.applyDelta(ops as any)

          try {
            const y = await applyDelta(docId, cid, origin, ops)
            console.log('applyDelta', y)
          } catch (e) {
            console.error('applyDelta', e)
          }

          doc.commit(origin)
          // const delta = doc.exportFrom(current)
          // emitter.emit('delta', { docId, delta })
        },
        subscribe(listener: (delta: HulyDelta) => void) {
          subscribe(docId, cid)
          doc.subscribe((batch: LoroEventBatch) => {
            console.log('lorobatch', batch)
            if (batch.origin === origin) return
            batch.events.forEach((event) => {
              if (event.path[0] === path) {
                const diff = event.diff
                if (diff.type === 'text') listener({ docId, ops: diff.diff })
              }
            })
          })
        },
      }
    },
  }
}

export function createPeer(peerId: bigint): HulyPeer {
  const docs = new Map<string, HulyDoc>()
  // emitter.on('delta', (delta) => {
  //   const doc = docs.get(delta.docId)
  //   if (doc) doc.import(delta.delta)
  // })
  return {
    peerId,
    getDoc(docId: string): HulyDoc {
      const doc = docs.get(docId)
      if (doc) {
        return doc
      } else {
        const newDoc = createDoc(peerId, docId)
        docs.set(docId, newDoc)
        return newDoc
      }
    },
  }
}
