export type Engine = {
  id: string
  name: string
  author: string
}

export type EnginesResponse = {
  engines: Engine[]
}

export type MatchCreateRequest = {
  white_engine_id: string
  black_engine_id: string
  time_control: {
    initial_ms: number
  }
}

export type MatchCreateResponse = {
  match_id: string
}

export type MatchResult = {
  result: string
  reason: string
}

export type MatchStatusResponse = {
  match_id: string
  status: 'running' | 'finished' | 'error'
  current_fen: string
  pgn: string
  clocks: {
    white_ms: number
    black_ms: number
  }
  result: MatchResult | null
}

export async function fetchEngines(): Promise<EnginesResponse> {
  const response = await fetch('/api/engines')
  if (!response.ok) {
    throw new Error('Failed to load engines')
  }
  return response.json()
}

export async function createMatch(payload: MatchCreateRequest): Promise<MatchCreateResponse> {
  const response = await fetch('/api/match', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(payload),
  })
  if (!response.ok) {
    throw new Error('Failed to create match')
  }
  return response.json()
}
