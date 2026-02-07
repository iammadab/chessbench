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

const sampleFen =
  'rnbqkbnr/pppppppp/8/8/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2'
const samplePgn = '1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7'

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

function MoveList({ pgn }: { pgn: string }) {
  const rows = parsePgnMoves(pgn)
  return (
    <div className="moves">
      <div className="moves-header">
        <h2>Moves</h2>
        <span>{rows.length} moves</span>
      </div>
      <div className="moves-list">
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

function App() {
  return (
    <div className="app">
      <header className="app-header">
        <div>
          <p className="eyebrow">Chessbench UI</p>
          <h1>Engine match monitor</h1>
        </div>
        <div className="status-pill">
          <span className="status-dot" />
          Sample data
        </div>
      </header>
      <main className="app-main">
        <section className="board-panel">
          <Chessboard fen={sampleFen} />
          <div className="board-caption">
            <span className="caption-title">Current position</span>
            <span className="caption-meta">FEN-driven render</span>
          </div>
        </section>
        <section className="moves-panel">
          <MoveList pgn={samplePgn} />
        </section>
      </main>
    </div>
  )
}

export default App
