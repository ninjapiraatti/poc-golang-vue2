import { reactive } from 'vue'
import { api } from '@root/api.js'
//import router from '@root/router.js'

const state = reactive({
  counter: 666,
  loggeduser: JSON.parse(localStorage.getItem('user')),
  colorScheme: getComputedStyle(document.documentElement).getPropertyValue('--color-scheme').trim(),
})

const methods = {
  increase() {
    state.counter++
  },

  decrease() {
    state.counter--
  },

	async logout() {
		try {
			await api.users.log.out()
			await this.setUser(null)
			router.push({ name: 'login' })
		} catch (error) {
			console.warn(`Logout failed: ${error.message}`)
			return false
		}
		return true
	},

  async login(data) {
		try {
			const userId = await api.users.log.in(data)
			if (userId) await this.setUser(userId)
		} catch (error) {
			console.warn(`Login failed: ${error.message}`)
			return false
		}
		return true
	},

  async setUser(data) {
		if (typeof data == 'string') {
			try {
				const [ user ] = await Promise.all([
					api.users.get({ id: data }),
				])

				data = user
			} catch (error) {
				data = null
			}
		}
    state.loggeduser = data
		if (data) {
			localStorage.setItem('user', JSON.stringify(data))
		} else {
			localStorage.removeItem('user')
		}
	},
}

export default {
  state,
  methods
}