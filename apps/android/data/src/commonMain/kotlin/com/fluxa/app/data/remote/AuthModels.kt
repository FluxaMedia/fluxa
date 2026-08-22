package com.fluxa.app.data.remote

data class LoginRequest(val email: String, val password: String)

data class AuthRequest(val authKey: String)

data class AuthResponse(val result: AuthResult)

data class AuthResponseWrapper(val user: AuthUser)

data class AuthResult(val user: AuthUser)

data class AuthUser(val id: String, val email: String, val authKey: String)
