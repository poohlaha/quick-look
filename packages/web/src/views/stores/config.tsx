/**
 * @fileOverview store
 * @date 2023-04-12
 * @author poohlaha
 */
import commonStore from './base/common.store'
import homeStore from './modules/home.store'

export function createStore() {
  return {
    commonStore,
    homeStore,
  }
}

export const store = createStore()
export type Stores = ReturnType<typeof createStore>
