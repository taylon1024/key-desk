import type { Health, Variable, VariableInput } from './types.ts'

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      Accept: 'application/json',
      ...(init?.body ? { 'Content-Type': 'application/json' } : {}),
      ...init?.headers,
    },
  })

  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null
    throw new Error(body?.error ?? `请求失败（${response.status}）`)
  }

  if (response.status === 204) {
    return undefined as T
  }

  const contentType = response.headers.get('content-type') ?? ''
  if (contentType.includes('application/json')) {
    return (await response.json()) as T
  }

  return (await response.text()) as T
}

export function fetchHealth(): Promise<Health> {
  return request<Health>('/api/health')
}

export function fetchVariables(scope?: string): Promise<Variable[]> {
  const query = scope ? `?scope=${encodeURIComponent(scope)}` : ''
  return request<Variable[]>(`/api/variables${query}`)
}

export function createVariable(input: VariableInput): Promise<Variable> {
  return request<Variable>('/api/variables', {
    method: 'POST',
    body: JSON.stringify(input),
  })
}

export function updateVariable(id: number, input: VariableInput): Promise<Variable> {
  return request<Variable>(`/api/variables/${id}`, {
    method: 'PUT',
    body: JSON.stringify(input),
  })
}

export function deleteVariable(id: number): Promise<void> {
  return request<void>(`/api/variables/${id}`, { method: 'DELETE' })
}

export function exportVariables(scope?: string): Promise<string> {
  const query = scope ? `?scope=${encodeURIComponent(scope)}` : ''
  return request<string>(`/api/export${query}`)
}
