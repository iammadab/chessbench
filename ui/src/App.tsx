import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { createMatch, fetchEngines } from './api'
import type { Engine, MatchResult } from './api'
import { createMatchStream } from './sse'
import type { ClockEvent, MatchStartedEvent, MoveEvent, ResultEvent } from './sse'
import { createMockMatch, createMockStream, getMockEngines } from './mock'
import bB from './assets/pieces/bB.svg'
import bK from './assets/pieces/bK.svg'
import bN from './assets/pieces/bN.svg'
import bP from './assets/pieces/bP.svg'
import bQ from './assets/pieces/bQ.svg'
import bR from './assets/pieces/bR.svg'
import wB from './assets/pieces/wB.svg'
import wK from './assets/pieces/wK.svg'
import wN from './assets/pieces/wN.svg'
import wP from './assets/pieces/wP.svg'
import wQ from './assets/pieces/wQ.svg'
import wR from './assets/pieces/wR.svg'
import './App.css'

const startFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
const defaultInitialMs = 5 * 60 * 1000
const defaultMinutes = 5
const defaultSeconds = 0

type Square = string | null

const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']

function parseFen(fen: string): Square[][] {
  const [placement] = fen.split(' ')
  const ranks = placement.split('/')
  return ranks.map((rank) => {
    const row: Square[] = []
    for (const char of rank) {
      if (char >= '1' && char <= '8') {
        const emptyCount = Number(char)
        for (let i = 0; i < emptyCount; i += 1) {
          row.push(null)
        }
      } else {
        row.push(char)
      }
    }
    return row
  })
}

function parsePgnMoves(pgn: string) {
  const tokens = pgn
    .replace(/\{[^}]*\}/g, '')
    .replace(/\([^)]*\)/g, '')
    .split(/\s+/)
    .filter(Boolean)

  const moves: string[] = []
  for (const token of tokens) {
    if (/^\d+\.+$/.test(token)) {
      continue
    }
    if (token === '1-0' || token === '0-1' || token === '1/2-1/2' || token === '*') {
      continue
    }
    moves.push(token)
  }

  const rows: { moveNumber: number; white: string; black: string }[] = []
  for (let i = 0; i < moves.length; i += 2) {
    rows.push({
      moveNumber: Math.floor(i / 2) + 1,
      white: moves[i] ?? '',
      black: moves[i + 1] ?? '',
    })
  }
  return rows
}

const pieceSrc: Record<string, string> = {
  p: bP,
  r: bR,
  n: bN,
  b: bB,
  q: bQ,
  k: bK,
  P: wP,
  R: wR,
  N: wN,
  B: wB,
  Q: wQ,
  K: wK,
}

function Piece({ code }: { code: string }) {
  const label = code.toUpperCase()
  return (
    <img
      className="piece"
      src={pieceSrc[code]}
      alt={label}
      loading="lazy"
      draggable={false}
    />
  )
}

function Chessboard({ fen }: { fen: string }) {
  const grid = parseFen(fen)
  return (
    <div className="board">
      <div className="board-inner">
        {grid.map((rank, rankIndex) =>
          rank.map((square, fileIndex) => {
            const isDark = (rankIndex + fileIndex) % 2 === 1
            const key = `${rankIndex}-${fileIndex}`
            return (
              <div
                key={key}
                className={`square ${isDark ? 'dark' : 'light'}`}
              >
                {square ? <Piece code={square} /> : null}
              </div>
            )
          }),
        )}
      </div>
      <div className="board-coords files">
        {files.map((file) => (
          <span key={file}>{file}</span>
        ))}
      </div>
      <div className="board-coords ranks">
        {[8, 7, 6, 5, 4, 3, 2, 1].map((rank) => (
          <span key={rank}>{rank}</span>
        ))}
      </div>
    </div>
  )
}

function uciToSquares(uci: string | null) {
  if (!uci || uci.length < 4) return null
  return {
    from: uci.slice(0, 2),
    to: uci.slice(2, 4),
  }
}

function coordsToSquare(fileIndex: number, rankIndex: number) {
  const file = files[fileIndex]
  const rank = 8 - rankIndex
  return `${file}${rank}`
}

function ChessboardWithHighlight({
  fen,
  lastMoveUci,
}: {
  fen: string
  lastMoveUci: string | null
}) {
  const grid = parseFen(fen)
  const lastMove = uciToSquares(lastMoveUci)
  return (
    <div className="board">
      <div className="board-inner">
        {grid.map((rank, rankIndex) =>
          rank.map((square, fileIndex) => {
            const isDark = (rankIndex + fileIndex) % 2 === 1
            const key = `${rankIndex}-${fileIndex}`
            const squareId = coordsToSquare(fileIndex, rankIndex)
            const isFrom = lastMove?.from === squareId
            const isTo = lastMove?.to === squareId
            return (
              <div
                key={key}
                className={`square ${isDark ? 'dark' : 'light'}${isFrom ? ' highlight-from' : ''}${isTo ? ' highlight-to' : ''}`}
              >
                {square ? <Piece code={square} /> : null}
              </div>
            )
          }),
        )}
      </div>
      <div className="board-coords files">
        {files.map((file) => (
          <span key={file}>{file}</span>
        ))}
      </div>
      <div className="board-coords ranks">
        {[8, 7, 6, 5, 4, 3, 2, 1].map((rank) => (
          <span key={rank}>{rank}</span>
        ))}
      </div>
    </div>
  )
}

function MoveList({ pgn }: { pgn: string }) {
  const rows = parsePgnMoves(pgn)
  const listRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    if (!listRef.current) return
    listRef.current.scrollTop = listRef.current.scrollHeight
  }, [rows.length])
  return (
    <div className="moves">
      <div className="moves-header">
        <h2>Moves</h2>
        <span>{rows.length} moves</span>
      </div>
      <div className="moves-list" ref={listRef}>
        {rows.map((row) => (
          <div className="move-row" key={row.moveNumber}>
            <span className="move-number">{row.moveNumber}.</span>
            <span className="move-san white">{row.white}</span>
            <span className="move-san black">{row.black}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

type MatchStatus = 'idle' | 'loading' | 'connecting' | 'running' | 'finished' | 'error' | 'reconnecting'

function formatEngineLabel(engine?: Engine) {
  if (!engine) return 'Unselected'
  return `${engine.name} (${engine.author})`
}

function formatClock(ms: number) {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000))
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}:${seconds.toString().padStart(2, '0')}`
}

function App() {
  const [engines, setEngines] = useState<Engine[]>([])
  const [matchId, setMatchId] = useState<string | null>(null)
  const [fen, setFen] = useState(startFen)
  const [pgn, setPgn] = useState('')
  const [clocks, setClocks] = useState<ClockEvent | null>(null)
  const [result, setResult] = useState<MatchResult | null>(null)
  const [status, setStatus] = useState<MatchStatus>('idle')
  const [error, setError] = useState<string | null>(null)
  const [lastMoveUci, setLastMoveUci] = useState<string | null>(null)
  const [useMocks, setUseMocks] = useState(true)
  const [whiteEngineId, setWhiteEngineId] = useState('')
  const [blackEngineId, setBlackEngineId] = useState('')
  const [minutes, setMinutes] = useState(defaultMinutes)
  const [seconds, setSeconds] = useState(defaultSeconds)
  const streamRef = useRef<ReturnType<typeof createMatchStream> | null>(null)
  const reconnectTimerRef = useRef<number | null>(null)
  const mockStreamRef = useRef<ReturnType<typeof createMockStream> | null>(null)

  const statusLabel = useMemo(() => {
    if (status === 'loading') return 'Loading engines'
    if (status === 'connecting') return 'Starting match'
    if (status === 'running') return 'Match running'
    if (status === 'finished') return result ? `Result ${result.result}` : 'Match finished'
    if (status === 'reconnecting') return 'Reconnecting stream'
    if (status === 'error') return 'Connection error'
    return 'Idle'
  }, [result, status])

  const closeStream = useCallback(() => {
    if (reconnectTimerRef.current) {
      window.clearTimeout(reconnectTimerRef.current)
      reconnectTimerRef.current = null
    }
    streamRef.current?.close()
    streamRef.current = null
    mockStreamRef.current?.close()
    mockStreamRef.current = null
  }, [])

  const openStream = useCallback(
    (id: string) => {
      closeStream()
      streamRef.current = createMatchStream(id, {
        onOpen: () => {
          setStatus((current) => (current === 'reconnecting' ? 'running' : current))
        },
        onMatchStarted: (data: MatchStartedEvent) => {
          setStatus('running')
          setFen(data.start_fen || startFen)
          setPgn('')
          setResult(null)
        },
        onClock: (data: ClockEvent) => {
          setClocks(data)
        },
        onMove: (data: MoveEvent) => {
          setFen(data.fen)
          setPgn(data.pgn)
          setLastMoveUci(data.uci)
        },
        onResult: (data: ResultEvent) => {
          setResult(data)
          setStatus('finished')
          closeStream()
        },
        onError: () => {
          setStatus('reconnecting')
          closeStream()
          reconnectTimerRef.current = window.setTimeout(() => openStream(id), 1500)
        },
      })
    },
    [closeStream],
  )

  const resetMatch = useCallback(() => {
    closeStream()
    setMatchId(null)
    setFen(startFen)
    setPgn('')
    setClocks(null)
    setResult(null)
    setLastMoveUci(null)
    setStatus('idle')
    setError(null)
  }, [closeStream])

  useEffect(() => {
    let active = true
    setStatus('loading')
    const enginesPromise = useMocks ? getMockEngines() : fetchEngines()
    enginesPromise
      .then((data) => {
        if (!active) return
        setEngines(data.engines)
        setWhiteEngineId(data.engines[0]?.id ?? '')
        setBlackEngineId(data.engines[1]?.id ?? data.engines[0]?.id ?? '')
        setStatus('idle')
      })
      .catch((err: Error) => {
        if (!active) return
        setStatus('error')
        setError(err.message)
      })
    return () => {
      active = false
    }
  }, [useMocks])

  useEffect(() => {
    return () => {
      closeStream()
    }
  }, [closeStream])

  const initialMs = useMemo(() => {
    const safeMinutes = Number.isFinite(minutes) ? Math.max(0, minutes) : 0
    const safeSeconds = Number.isFinite(seconds) ? Math.min(59, Math.max(0, seconds)) : 0
    return (safeMinutes * 60 + safeSeconds) * 1000
  }, [minutes, seconds])

  const whiteEngine = useMemo(
    () => engines.find((engine) => engine.id === whiteEngineId),
    [engines, whiteEngineId],
  )
  const blackEngine = useMemo(
    () => engines.find((engine) => engine.id === blackEngineId),
    [engines, blackEngineId],
  )

  const handleStart = useCallback(() => {
    if (!whiteEngineId || !blackEngineId) {
      setError('Select both engines to start a match.')
      return
    }
    if (initialMs <= 0) {
      setError('Time control must be greater than 0 seconds.')
      return
    }
    setError(null)
    setStatus('connecting')
    closeStream()
    setPgn('')
    setResult(null)

    if (useMocks) {
      createMockMatch()
        .then((data) => {
          setMatchId(data.match_id)
          mockStreamRef.current = createMockStream(initialMs, {
            onOpen: () => setStatus('running'),
            onMatchStarted: (data: MatchStartedEvent) => {
              setStatus('running')
              setFen(data.start_fen || startFen)
              setPgn('')
              setResult(null)
            },
            onClock: (data: ClockEvent) => setClocks(data),
            onMove: (data: MoveEvent) => {
              setFen(data.fen)
              setPgn(data.pgn)
              setLastMoveUci(data.uci)
            },
            onResult: (data: ResultEvent) => {
              setResult(data)
              setStatus('finished')
              closeStream()
            },
          })
        })
        .catch((err: Error) => {
          setStatus('error')
          setError(err.message)
        })
      return
    }

    createMatch({
      white_engine_id: whiteEngineId,
      black_engine_id: blackEngineId,
      time_control: { initial_ms: initialMs },
    })
      .then((data) => {
        setMatchId(data.match_id)
        openStream(data.match_id)
      })
      .catch((err: Error) => {
        setStatus('error')
        setError(err.message)
      })
  }, [
    blackEngineId,
    closeStream,
    initialMs,
    openStream,
    useMocks,
    whiteEngineId,
  ])

  return (
    <div className="app">
      <header className="app-header">
        <div>
          <p className="eyebrow">Chessbench</p>
        </div>
        <div className="status-pill">
          <span className="status-dot" />
          {statusLabel}
        </div>
      </header>
      {error ? <p className="status-error">{error}</p> : null}
      <main className="app-main">
        <section className="board-panel">
          <div className="side-label top">
            <span className="side-tag">Black</span>
            <span className="side-name">{formatEngineLabel(blackEngine)}</span>
            <span className="side-clock">{formatClock(clocks?.black_ms ?? initialMs)}</span>
          </div>
          <ChessboardWithHighlight fen={fen || startFen} lastMoveUci={lastMoveUci} />
          <div className="side-label bottom">
            <span className="side-tag">White</span>
            <span className="side-name">{formatEngineLabel(whiteEngine)}</span>
            <span className="side-clock">{formatClock(clocks?.white_ms ?? initialMs)}</span>
          </div>
          <div className="board-caption">
            <span className="caption-meta">
              {matchId ? `Match ${matchId}` : engines.length ? `${engines.length} engines` : 'No engines'}
            </span>
          </div>
        </section>
        <section className="moves-panel">
          <section className="controls">
            <div className="control-group">
              <label htmlFor="white-engine">White</label>
              <select
                id="white-engine"
                value={whiteEngineId}
                onChange={(event) => setWhiteEngineId(event.target.value)}
                disabled={status === 'running' || status === 'connecting'}
              >
                {engines.map((engine) => (
                  <option key={engine.id} value={engine.id}>
                    {engine.name} ({engine.author})
                  </option>
                ))}
              </select>
            </div>
            <div className="control-group">
              <label htmlFor="black-engine">Black</label>
              <select
                id="black-engine"
                value={blackEngineId}
                onChange={(event) => setBlackEngineId(event.target.value)}
                disabled={status === 'running' || status === 'connecting'}
              >
                {engines.map((engine) => (
                  <option key={engine.id} value={engine.id}>
                    {engine.name} ({engine.author})
                  </option>
                ))}
              </select>
            </div>
            <div className="control-group time-group">
              <label>Time</label>
              <div className="time-inputs">
                <input
                  type="number"
                  min={0}
                  value={minutes}
                  onChange={(event) => setMinutes(Number(event.target.value))}
                  disabled={status === 'running' || status === 'connecting'}
                />
                <span>:</span>
                <input
                  type="number"
                  min={0}
                  max={59}
                  value={seconds}
                  onChange={(event) => setSeconds(Number(event.target.value))}
                  disabled={status === 'running' || status === 'connecting'}
                />
              </div>
            </div>
            <button
              className="start-button"
              onClick={handleStart}
              disabled={status === 'running' || status === 'connecting' || engines.length === 0}
            >
              Start match
            </button>
            <button
              className="stop-button"
              type="button"
              onClick={resetMatch}
              disabled={status !== 'running' && status !== 'connecting'}
            >
              Stop match
            </button>
            <button
              className="toggle-button"
              type="button"
              onClick={() => {
                resetMatch()
                setUseMocks((current) => !current)
              }}
            >
              {useMocks ? 'Mock data on' : 'Mock data off'}
            </button>
          </section>
          <MoveList pgn={pgn} />
        </section>
      </main>
    </div>
  )
}

export default App
