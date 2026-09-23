export type Variable = {
  id: number
  key: string
  value: string
  scope: string
  description: string
  is_secret: boolean
  created_at: string
  updated_at: string
}

export type VariableInput = {
  key: string
  value: string
  scope: string
  description: string
  is_secret: boolean
}

export type Health = {
  status: string
}
