import { Loro, type LoroEventBatch } from 'loro-crdt'
import mitt from 'mitt'
import { Op } from 'quill-delta'

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
  readonly origin?: Origin
}

interface HulyText {
  getContents(): Op[]
  update(delta: HulyDelta): void
  subscribe(listener: (delta: HulyDelta) => void): void
}

type RemoteDelta = {
  readonly docId: DocId
  readonly delta: Uint8Array
}

type Events = {
  delta: RemoteDelta
}
const emitter = mitt<Events>()

const after = { expand: 'after' as 'after' }

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
      const text = doc.getText(path)
      return {
        getContents() {
          return text.toDelta()
        },
        update(local: HulyDelta) {
          const current = doc.version()
          console.log('update', local.ops)
          text.applyDelta(local.ops as any)
          doc.commit(local.origin)
          const delta = doc.exportFrom(current)
          emitter.emit('delta', { docId, delta })
        },
        subscribe(listener: (delta: HulyDelta) => void) {
          doc.subscribe((batch: LoroEventBatch) => {
            console.log('lorobatch', batch)
            batch.events.forEach((event) => {
              if (event.path[0] === path) {
                const diff = event.diff
                if (diff.type === 'text')
                  listener({
                    docId,
                    ops: diff.diff,
                    origin: batch.origin,
                  })
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
  emitter.on('delta', (delta) => {
    const doc = docs.get(delta.docId)
    if (doc) doc.import(delta.delta)
  })
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
