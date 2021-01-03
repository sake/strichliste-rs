let baseUrl = "http://localhost:8080/api"
let settingsApi = () => j`${baseUrl}/settings`
let usersApi = () => j`${baseUrl}/user`
let userApi = (uid) => j`${baseUrl}/user/${uid}`
let userTxApi = (uid) => j`${baseUrl}/user/${uid}/transaction`