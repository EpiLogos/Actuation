import React from 'react'
import type { ClientContext } from '@deepseek-ai/dsh-client-runtime/client'
import type {
  ChatConversationViewNode,
  ConversationNodeContext,
  ConversationNodeDefinition,
} from '@deepseek-ai/dsh-client-runtime/client'
import type {} from '@deepseek-ai/dsh-client-ui-conversation/client'

export const inject = ['conversationEvents', 'slots']

export interface Series1QLInspectionData {
  readonly runId: string
  readonly condition: string
  readonly firstRecordIndex: number
  readonly lastRecordIndex: number
  readonly events: readonly {
    readonly recordIndex: number
    readonly channel: string
    readonly eventType: string
    readonly qlPosition?: string
    readonly relation?: string
  }[]
}

declare module '@deepseek-ai/dsh-client-ui-conversation/client' {
  interface ChatNodeDataMap {
    'series1-ql-inspection': Series1QLInspectionData
  }
}

interface State {
  readonly runId: string
  readonly condition: string
  readonly events: Series1QLInspectionData['events']
}

type PortableInspectionEvent = {
  readonly type: 'series1/portable-event'
  readonly seq: number
  readonly data: {
    readonly run_id: string
    readonly condition: string
    readonly portable_record_index: number
    readonly portable_channel: string
    readonly portable_event_type: string
    readonly event?: {
      readonly ql?: { readonly position?: string }
      readonly relation?: string
      readonly payload?: { readonly position?: string; readonly relation?: string }
    }
  }
}

function asPortable(event: unknown): PortableInspectionEvent | null {
  const candidate = event as Partial<PortableInspectionEvent>
  return candidate?.type === 'series1/portable-event' && candidate.data !== undefined
    ? candidate as PortableInspectionEvent
    : null
}

function item(event: PortableInspectionEvent): Series1QLInspectionData['events'][number] {
  return {
    recordIndex: event.data.portable_record_index,
    channel: event.data.portable_channel,
    eventType: event.data.portable_event_type,
    ...((event.data.event?.ql?.position ?? event.data.event?.payload?.position) === undefined
      ? {}
      : { qlPosition: event.data.event?.ql?.position ?? event.data.event?.payload?.position }),
    ...((event.data.event?.relation ?? event.data.event?.payload?.relation) === undefined
      ? {}
      : { relation: event.data.event?.relation ?? event.data.event?.payload?.relation }),
  }
}

/** Read-only Definition over the separate evidence-only inspection Session. */
export const series1QLInspectionDefinition: ConversationNodeDefinition<State> = {
  kind: 'series1-ql-inspection',
  target: 'chat',
  match(event) {
    const portable = asPortable(event)
    if (portable === null) return null
    return {
      id: portable.data.run_id,
      role: portable.data.portable_record_index === 0 ? 'start' : 'update',
    }
  },
  start(_context, match) {
    const portable = asPortable(match.event)
    if (portable === null) throw new Error('Series 1 QL inspection start requires a portable evidence event')
    return {
      runId: portable.data.run_id,
      condition: portable.data.condition,
      events: [item(portable)],
    }
  },
  update(context, match) {
    const portable = asPortable(match.event)
    if (portable === null) return context.state
    return { ...context.state, events: [...context.state.events, item(portable)] }
  },
  buildViewNode(context: ConversationNodeContext<State>): ChatConversationViewNode | null {
    if (context.start === undefined || context.state === undefined || context.state.events.length === 0) return null
    const events = context.state.events
    return {
      key: context.key,
      kind: 'series1-ql-inspection',
      id: context.id,
      target: 'chat',
      anchorSeq: context.start.event.seq,
      location: context.start.location,
      visibility: 'visible',
      data: {
        runId: context.state.runId,
        condition: context.state.condition,
        firstRecordIndex: events[0].recordIndex,
        lastRecordIndex: events[events.length - 1].recordIndex,
        events,
      } satisfies Series1QLInspectionData,
    }
  },
}

function InspectionPanel(props: any) {
  const data = props.data as Series1QLInspectionData
  const meaningful = data.events.filter(event => event.channel === 'runtime-semantic')
  return (
    <section data-series1-observer="true" aria-label="Series 1 QL inspection">
      <header>
        <strong>Series 1 · {data.condition}</strong>
        <span> · portable events {data.firstRecordIndex}–{data.lastRecordIndex}</span>
      </header>
      <ol>
        {meaningful.map(event => (
          <li key={event.recordIndex}>
            <code>#{event.recordIndex}</code>{' '}
            <span>{event.eventType}</span>
            {event.qlPosition === undefined ? null : <span> · {event.qlPosition}</span>}
            {event.relation === undefined ? null : <span> · {event.relation}</span>}
          </li>
        ))}
      </ol>
    </section>
  )
}

/**
 * DSH Web Client contribution. It registers only a Conversation read model and
 * keyed renderer. It has no Agent, Session append, RPC, model-context, composer,
 * followup, steer, inject or send dependency.
 */
export function apply(ctx: ClientContext): void {
  ctx.conversationEvents.register(series1QLInspectionDefinition)
  ctx.slots.inject('conversation.chat.node', () => ctx.slots.register({
    name: 'conversation.chat.node',
    key: 'series1-ql-inspection',
    inject: () => ({}),
  }, InspectionPanel))
}
