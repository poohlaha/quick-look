/**
 * @fileOverview base store, all store muse extends this store
 * @date 2023-04-12
 * @author poohlaha
 */
import {action, observable} from 'mobx'
import Utils from '@utils/utils'
import { COMMON, TOAST } from '@utils/base'

export default class BaseStore {
  @observable loading: boolean = false

  /**
   * 设置属性
   */
  @action
  setProperty = (property: any, value: any) => {
    // @ts-ignore
    this[property] = value
  }

  /**
   * 获取属性
   */
  @action
  getProperty = (property: any) => {
    // @ts-ignore
    return this[property]
  }

  @action
  analysisResult = (result: {[K: string]: any} = {}, errMsg: string = '') => {
    if (Utils.isObjectNull(result)) {
      TOAST.show({ message: errMsg || COMMON.getLanguageText('ERROR_MESSAGE'), type: 4 })
      return {}
    }

    /*
    // 解除多次转义，将其还原为单次转义的 JSON 字符串
    let newData = data
      .replace(/^"/, '') // 去掉开头的双引号
      .replace(/"$/, '') // 去掉末尾的双引号
      .replace(/\\"/g, '"') // 将多次转义的双引号还原为单次转义的双引号

    newData = newData.replace(/\\n/g, '\n')
    console.log('newData: ', newData)
     */

    let error = result.error || ''
    if (!Utils.isBlank(error) || result.code !== 200) {
      TOAST.show({ message: errMsg || error || COMMON.getLanguageText('ERROR_MESSAGE'), type: 4 })
      return {}
    }

    let content = result.body || {}
    if (typeof content === 'string') {
      content = content
        .replace(/^"/, '') // 去掉开头的双引号
        .replace(/"$/, '') // 去掉末尾的双引号
        .replace(/\\"/g, '"') // 将多次转义的双引号还原为单次转义的双引号
        .replace(/\\r/g, '\n')
        .replace(/\\n/g, '\n')
        .replace(/↵/g, '')
        .replace(/\\t/g, '  ')
        .replace(/\t/g, '  ')
    }

    return content || {}
  }
}
