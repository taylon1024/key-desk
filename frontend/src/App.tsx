import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import {
  createVariable,
  deleteVariable,
  exportVariables,
  fetchHealth,
  fetchVariables,
  updateVariable,
} from './api.ts'
import type { Variable, VariableInput } from './types.ts'

const emptyForm: VariableInput = {
  key: '',
  value: '',
  scope: 'default',
  description: '',
  is_secret: false,
}

export default function App() {
  const [variables, setVariables] = useState<Variable[]>([])
  const [form, setForm] = useState<VariableInput>(emptyForm)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [scopeFilter, setScopeFilter] = useState('all')
  const [revealed, setRevealed] = useState<number[]>([])
  const [online, setOnline] = useState(false)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const loadSeq = useRef(0)

  async function load() {
    const seq = ++loadSeq.current
    const next = await fetchVariables()
    if (seq !== loadSeq.current) return
    setVariables(next)
  }

  useEffect(() => {
    let active = true
    async function boot() {
      try {
        await fetchHealth()
        if (!active) return
        setOnline(true)
        await load()
      } catch (err) {
        if (!active) return
        setOnline(false)
        setError(err instanceof Error ? err.message : '无法连接后端')
      } finally {
        if (active) setLoading(false)
      }
    }
    void boot()
    return () => {
      active = false
    }
  }, [])

  const scopes = useMemo(() => {
    const names = new Set(variables.map((variable) => variable.scope))
    if (form.scope.trim()) names.add(form.scope.trim())
    return [...names].sort()
  }, [variables, form.scope])

  const visible = useMemo(
    () =>
      scopeFilter === 'all'
        ? variables
        : variables.filter((variable) => variable.scope === scopeFilter),
    [variables, scopeFilter],
  )

  function updateField<K extends keyof VariableInput>(key: K, value: VariableInput[K]) {
    setForm((current) => ({ ...current, [key]: value }))
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSaving(true)
    setError(null)
    try {
      if (editingId === null) {
        await createVariable(form)
      } else {
        await updateVariable(editingId, form)
      }
      setForm(emptyForm)
      setEditingId(null)
      await load()
    } catch (err) {
      setError(err instanceof Error ? err.message : '保存失败')
    } finally {
      setSaving(false)
    }
  }

  function startEdit(variable: Variable) {
    setEditingId(variable.id)
    setForm({
      key: variable.key,
      value: variable.value,
      scope: variable.scope,
      description: variable.description,
      is_secret: variable.is_secret,
    })
  }

  function cancelEdit() {
    setEditingId(null)
    setForm(emptyForm)
  }

  async function onDelete(variable: Variable) {
    const confirmed = window.confirm(`删除 ${variable.scope} / ${variable.key}？`)
    if (!confirmed) return
    setError(null)
    try {
      await deleteVariable(variable.id)
      if (editingId === variable.id) cancelEdit()
      await load()
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除失败')
    }
  }

  async function onExport() {
    setError(null)
    try {
      const scope = scopeFilter === 'all' ? undefined : scopeFilter
      const text = await exportVariables(scope)
      const blob = new Blob([text], { type: 'text/plain;charset=utf-8' })
      const url = URL.createObjectURL(blob)
      const link = document.createElement('a')
      link.href = url
      link.download = `${scope ?? 'all'}.env`
      link.click()
      URL.revokeObjectURL(url)
    } catch (err) {
      setError(err instanceof Error ? err.message : '导出失败')
    }
  }

  async function copyValue(variable: Variable) {
    try {
      await navigator.clipboard.writeText(variable.value)
    } catch (err) {
      setError(err instanceof Error ? err.message : '复制失败')
    }
  }

  function toggleReveal(id: number) {
    setRevealed((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    )
  }

  return (
    <div className="app">
      <header className="topbar">
        <div>
          <p className="eyebrow">Envman</p>
          <h1>环境变量</h1>
        </div>
        <div className="topbar-actions">
          <span className={online ? 'status ok' : 'status bad'}>
            {online ? '服务已连接' : '服务未连接'}
          </span>
          <button type="button" className="ghost" onClick={() => void onExport()} disabled={!online}>
            导出 .env
          </button>
        </div>
      </header>

      {error ? <p className="banner">{error}</p> : null}

      <form className="composer" onSubmit={(event) => void onSubmit(event)}>
        <div className="composer-head">
          <h2>{editingId === null ? '新增变量' : `编辑 #${editingId}`}</h2>
          {editingId !== null ? (
            <button type="button" className="text" onClick={cancelEdit}>
              取消
            </button>
          ) : null}
        </div>
        <div className="fields">
          <label>
            变量名
            <input
              value={form.key}
              onChange={(event) => updateField('key', event.target.value)}
              placeholder="DATABASE_URL"
              autoCapitalize="off"
              spellCheck={false}
              required
            />
          </label>
          <label>
            作用域
            <input
              value={form.scope}
              onChange={(event) => updateField('scope', event.target.value)}
              list="scopes"
              placeholder="default"
              required
            />
          </label>
          <label className="wide">
            值
            <input
              value={form.value}
              onChange={(event) => updateField('value', event.target.value)}
              type={form.is_secret ? 'password' : 'text'}
              spellCheck={false}
            />
          </label>
          <label className="wide">
            说明
            <input
              value={form.description}
              onChange={(event) => updateField('description', event.target.value)}
              placeholder="可选"
            />
          </label>
        </div>
        <div className="composer-foot">
          <label className="check">
            <input
              type="checkbox"
              checked={form.is_secret}
              onChange={(event) => updateField('is_secret', event.target.checked)}
            />
            敏感值
          </label>
          <button type="submit" disabled={saving || !online}>
            {saving ? '保存中…' : editingId === null ? '添加' : '保存'}
          </button>
        </div>
        <datalist id="scopes">
          {scopes.map((scope) => (
            <option key={scope} value={scope} />
          ))}
        </datalist>
      </form>

      <section className="list">
        <div className="toolbar">
          <label>
            筛选作用域
            <select value={scopeFilter} onChange={(event) => setScopeFilter(event.target.value)}>
              <option value="all">全部</option>
              {scopes.map((scope) => (
                <option key={scope} value={scope}>
                  {scope}
                </option>
              ))}
            </select>
          </label>
          <span className="count">{loading ? '加载中' : `${visible.length} 条`}</span>
        </div>

        {visible.length === 0 && !loading ? (
          <p className="empty">这个范围内还没有变量。</p>
        ) : (
          <ul>
            {visible.map((variable) => {
              const shown = !variable.is_secret || revealed.includes(variable.id)
              return (
                <li key={variable.id}>
                  <div className="item-main">
                    <div className="item-title">
                      <code>{variable.key}</code>
                      <span className="scope">{variable.scope}</span>
                      {variable.is_secret ? <span className="secret">敏感</span> : null}
                    </div>
                    <p className="value">{shown ? variable.value || '（空）' : '••••••••'}</p>
                    {variable.description ? <p className="desc">{variable.description}</p> : null}
                  </div>
                  <div className="item-actions">
                    {variable.is_secret ? (
                      <button type="button" className="text" onClick={() => toggleReveal(variable.id)}>
                        {shown ? '隐藏' : '显示'}
                      </button>
                    ) : null}
                    <button type="button" className="text" onClick={() => void copyValue(variable)}>
                      复制
                    </button>
                    <button type="button" className="text" onClick={() => startEdit(variable)}>
                      编辑
                    </button>
                    <button type="button" className="text danger" onClick={() => void onDelete(variable)}>
                      删除
                    </button>
                  </div>
                </li>
              )
            })}
          </ul>
        )}
      </section>
    </div>
  )
}
