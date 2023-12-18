/**
 * @fileOverview base store, all store muse extends this store
 * @date 2023-04-12
 * @author poohlaha
 */
import { action, observable } from 'mobx'
import Utils from '@utils/utils'
import { COMMON, TOAST } from '@utils/base'

export default class BaseStore {
  @observable currentPage: number = 1
  @observable pageSize: number = 10
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
  analysisResult = (result: { [K: string]: any } = {}, errMsg: string = '') => {
    if (Utils.isObjectNull(result)) {
      TOAST.show({ message: errMsg || COMMON.getLanguageText('ERROR_MESSAGE'), type: 4 })
      return
    }

    let error = result.error || ''
    if (!Utils.isBlank(error) || result.code !== 200) {
      TOAST.show({ message: error || errMsg || COMMON.getLanguageText('ERROR_MESSAGE'), type: 4 })
      return
    }

    let content = result.body || ''
    const suffixProps = result.suffixProps || {}

    try {
      let contents = JSON.parse(content)
      if (Array.isArray(contents)) {
        return contents || []
      }
    } catch (e) {
      console.log('body not array !')
    }

    if (!Utils.isBlank(content)) {
      let fileProps = result.fileProps || {}
      let suffix = fileProps.suffix || ''
      let imageSuffixes = (result.imageSuffixes || '').split(',') || []
      let isImage = false
      if (imageSuffixes.includes(suffix) && imageSuffixes.length > 0) {
        isImage = true
      }

      if (!isImage) {
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

      return content
    }

    // 压缩包
    if (suffixProps.type === 'archive' || suffixProps.type === 'dir') {
      return result.fileProps || {}
    }

    return ''
  }
}
