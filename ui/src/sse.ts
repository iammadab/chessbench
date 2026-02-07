export type MatchStartedEvent = {
  match_id: string
  start_fen: string
}

export type ClockEvent = {
  white_ms: number
  black_ms: number
}

export type MoveEvent = {
  ply: number
  uci: string
  san: string
  fen: string
  pgn: string
}

export type ResultEvent = {
  result: string
  reason: string
}

export type StreamHandlers = {
  onOpen?: () => void
  onMatchStarted?: (data: MatchStartedEvent) => void
  onClock?: (data: ClockEvent) => void
  onMove?: (data: MoveEvent) => void
  onResult?: (data: ResultEvent) => void
  onError?: () => void
}

const parseEvent = <T,>(event: MessageEvent): T => JSON.parse(event.data) as T

export function createMatchStream(matchId: string, handlers: StreamHandlers) {
  const source = new EventSource(`/api/match/${matchId}/stream`)

  source.onopen = () => {
    handlers.onOpen?.()
  }

  source.addEventListener('match_started', (event) => {
    handlers.onMatchStarted?.(parseEvent<MatchStartedEvent>(event as MessageEvent))
  })

  source.addEventListener('clock', (event) => {
    handlers.onClock?.(parseEvent<ClockEvent>(event as MessageEvent))
  })

  source.addEventListener('move', (event) => {
    handlers.onMove?.(parseEvent<MoveEvent>(event as MessageEvent))
  })

  source.addEventListener('result', (event) => {
    handlers.onResult?.(parseEvent<ResultEvent>(event as MessageEvent))
  })

  source.onerror = () => {
    handlers.onError?.()
  }

  return {
    close: () => source.close(),
  }
}
