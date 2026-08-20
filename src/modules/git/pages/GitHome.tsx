import { useState } from 'react'
import { checkGitRepository } from '../services/gitServices'

export default function GitHome() {
  const [path, setPath] = useState('')
  const [isRepository, setIsRepository] = useState<boolean | null>(null)

  async function handleCheckRepository() {
    const result = await checkGitRepository(path)

    setIsRepository(result)
  }

  return (
    <main>
      <h1>DEWRENCH</h1>

      <p>First Git Test</p>

      <input
        type="text"
        value={path}
        placeholder="C:\Users\user\Desktop\meu-projeto"
        onChange={(event) => setPath(event.target.value)}
      />

      <button onClick={handleCheckRepository}>
        Check repository
      </button>

      {isRepository === true && (
        <p>Git repository detected ✅</p>
      )}

      {isRepository === false && (
        <p>Git repository not detected ❌</p>
      )}
    </main>
  )
}