import type { Engine, EnginesResponse, MatchCreateResponse } from './api'
import type { StreamHandlers } from './sse'

const startFen = 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'

export const mockEngines: Engine[] = [
  { id: 'stockfish-16', name: 'Stockfish 16', author: 'SF Team' },
  { id: 'lc0-0.30', name: 'Leela Chess Zero', author: 'Lc0 Team' },
  { id: 'ethereal-14', name: 'Ethereal 14', author: 'Andrew Grant' },
]

const mockMoves = [
  { uci: 'e2e4', san: 'e4' },
  { uci: 'c7c5', san: 'c5' },
  { uci: 'g1f3', san: 'Nf3' },
  { uci: 'd7d6', san: 'd6' },
  { uci: 'd2d4', san: 'd4' },
  { uci: 'c5d4', san: 'cxd4' },
  { uci: 'f3d4', san: 'Nxd4' },
  { uci: 'g8f6', san: 'Nf6' },
  { uci: 'b1c3', san: 'Nc3' },
  { uci: 'a7a6', san: 'a6' },
  { uci: 'f1e2', san: 'Be2' },
  { uci: 'e7e6', san: 'e6' },
  { uci: 'e1g1', san: 'O-O' },
  { uci: 'f8e7', san: 'Be7' },
  { uci: 'f2f4', san: 'f4' },
  { uci: 'e8g8', san: 'O-O' },
  { uci: 'c1e3', san: 'Be3' },
  { uci: 'd8c7', san: 'Qc7' },
  { uci: 'a2a4', san: 'a4' },
  { uci: 'b8c6', san: 'Nc6' },
]

type BoardSquare = string | null

const files = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']

function createStartBoard(): BoardSquare[][] {
  return [
    ['r', 'n', 'b', 'q', 'k', 'b', 'n', 'r'],
    ['p', 'p', 'p', 'p', 'p', 'p', 'p', 'p'],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
    [null, null, null, null, null, null, null, null],
    ['P', 'P', 'P', 'P', 'P', 'P', 'P', 'P'],
    ['R', 'N', 'B', 'Q', 'K', 'B', 'N', 'R'],
  ]
}

function squareToCoords(square: string) {
  const file = files.indexOf(square[0])
  const rank = Number(square[1])
  return { row: 8 - rank, col: file }
}

function applyUciMove(board: BoardSquare[][], uci: string) {
  const from = uci.slice(0, 2)
  const to = uci.slice(2, 4)
  const promotion = uci.length > 4 ? uci[4] : null
  const { row: fromRow, col: fromCol } = squareToCoords(from)
  const { row: toRow, col: toCol } = squareToCoords(to)
  const piece = board[fromRow][fromCol]
  if (!piece) return

  board[fromRow][fromCol] = null

  if (piece === 'K' && from === 'e1' && to === 'g1') {
    board[7][5] = board[7][7]
    board[7][7] = null
  }
  if (piece === 'K' && from === 'e1' && to === 'c1') {
    board[7][3] = board[7][0]
    board[7][0] = null
  }
  if (piece === 'k' && from === 'e8' && to === 'g8') {
    board[0][5] = board[0][7]
    board[0][7] = null
  }
  if (piece === 'k' && from === 'e8' && to === 'c8') {
    board[0][3] = board[0][0]
    board[0][0] = null
  }

  let movedPiece = piece
  if (promotion) {
    movedPiece = piece === piece.toUpperCase() ? promotion.toUpperCase() : promotion.toLowerCase()
  }
  board[toRow][toCol] = movedPiece
}

function boardToPlacement(board: BoardSquare[][]) {
  return board
    .map((rank) => {
      let empties = 0
      let line = ''
      rank.forEach((square) => {
        if (!square) {
          empties += 1
          return
        }
        if (empties > 0) {
          line += String(empties)
          empties = 0
        }
        line += square
      })
      if (empties > 0) {
        line += String(empties)
      }
      return line
    })
    .join('/')
}

export function getMockEngines(): Promise<EnginesResponse> {
  return Promise.resolve({ engines: mockEngines })
}

export function createMockMatch(): Promise<MatchCreateResponse> {
  return Promise.resolve({ match_id: `mock-${Date.now()}` })
}

export function createMockStream(initialMs: number, handlers: StreamHandlers) {
  let moveIndex = 0
  let whiteMs = initialMs
  let blackMs = initialMs
  let closed = false
  let sideToMove: 'w' | 'b' = 'w'
  const board = createStartBoard()
  const pgnMoves: string[] = []

  handlers.onOpen?.()
  handlers.onMatchStarted?.({ match_id: 'mock', start_fen: startFen })

  const clockTimer = window.setInterval(() => {
    if (closed) return
    whiteMs = Math.max(0, whiteMs - 200)
    blackMs = Math.max(0, blackMs - 200)
    handlers.onClock?.({ white_ms: whiteMs, black_ms: blackMs })
  }, 200)

  const moveTimer = window.setInterval(() => {
    if (closed) return
    const move = mockMoves[moveIndex]
    if (!move) {
      handlers.onResult?.({ result: '1-0', reason: 'checkmate' })
      closed = true
      window.clearInterval(clockTimer)
      window.clearInterval(moveTimer)
      return
    }

    applyUciMove(board, move.uci)
    if (moveIndex % 2 === 0) {
      pgnMoves.push(`${Math.floor(moveIndex / 2) + 1}. ${move.san}`)
    } else {
      pgnMoves[pgnMoves.length - 1] += ` ${move.san}`
    }
    sideToMove = sideToMove === 'w' ? 'b' : 'w'
    const placement = boardToPlacement(board)
    const fen = `${placement} ${sideToMove} - - 0 1`

    handlers.onMove?.({
      ply: moveIndex + 1,
      uci: move.uci,
      san: move.san,
      fen,
      pgn: pgnMoves.join(' '),
    })
    moveIndex += 1
  }, 1200)

  return {
    close: () => {
      closed = true
      window.clearInterval(clockTimer)
      window.clearInterval(moveTimer)
    },
  }
}
