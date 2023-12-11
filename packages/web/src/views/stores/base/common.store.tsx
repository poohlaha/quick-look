/**
 * @fileOverview common store
 * @date 2023-04-12
 * @author poohlaha
 */
import { observable, action } from 'mobx'
import { CONSTANT } from '@config/index'
import Utils from '@utils/utils'
import BaseStore from '../base/base.store'

class CommonStore extends BaseStore {
  @observable skin = CONSTANT.SKINS[1] // 皮肤
  @observable socket: WebSocket | null = null // web socket
  @observable data: { [K: string]: any } = {} // 接收的数据

  constructor() {
    super()
    // this.initSocket()
  }

  /**
   * 切换皮肤
   * @param index
   */
  @action
  onSkinChange(index: number = -1) {
    if (index === -1) {
      this.skin = this.skin === CONSTANT.SKINS[1] ? CONSTANT.SKINS[0] : CONSTANT.SKINS[1]
    } else {
      this.skin = CONSTANT.SKINS[index]
    }
  }
}

export default new CommonStore()
