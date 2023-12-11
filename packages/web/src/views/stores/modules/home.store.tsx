/**
 * @fileOverview dashboard store
 * @date 2023-07-05
 * @author poohlaha
 */
import { observable, action } from 'mobx'
import BaseStore from '../base/base.store'
import { invoke } from '@tauri-apps/api/primitives'
import { info } from '@tauri-apps/plugin-log'
import {TOAST} from '@utils/base'
import Utils from '@utils/utils'

class HomeStore extends BaseStore {
  @observable fileName = '' // 文件名称
  @observable content: string | {[K: string]: any} = {} // 文件内容
  @observable suffixProps: Array<string> = [] // 图片后续列表
  @observable imageProps: { [K: string]: number | string} = {} // 图片属性

  @action
  reset() {
    this.fileName = ''
    this.content = ''
    this.loading = false
  }

  /**
   * 获取系统信息
   */
  @action
  async readFile(file: File | null = null, fileProps: {[K: string]: string} = {}) {
    try {
      this.imageProps = {}
      this.loading = true
      let result: {[K: string]: any} = {}

      if (file !== null) {
        this.fileName = file.name || ''
        let buffer: ArrayBuffer = await file.arrayBuffer()
        result = await invoke('file_handler', buffer, { headers: { fileName: encodeURIComponent(this.fileName) }})
      } else if (!Utils.isObjectNull(fileProps)) {
        console.log('fileProps:', fileProps)
        await info(`fileProps: ${JSON.stringify(fileProps)}`)

        this.fileName = fileProps.name || ''
        let filePath = fileProps.path || ''
        if (Utils.isBlank(filePath)) {
          console.log('file path is empty !')
          await info('file path is empty !')
          this.loading = false
          return
        }

        result = await invoke('file_handler', { filePath }, { headers: { fileName: encodeURIComponent(this.fileName) }})
      }

      console.log('result:', result)
      this.loading = false

      this.suffixProps = result.suffixProps || []
      await info(`suffixProps: ${JSON.stringify(this.suffixProps)}`)
      console.log('suffixProps:', this.suffixProps)

      this.content = this.analysisResult(result, `读取文件 ${this.fileName} 失败!`)
      if (this.content === undefined || this.content === null) {
        this.reset()
        return
      }
      // await info(`content: ${JSON.stringify(this.content)}`)

      // this.getImageProps(file)
    } catch (err: any) {
      this.reset()
      console.error('read file error !', err)
      TOAST.show({ message: `读取文件 ${this.fileName} 失败!`, type: 4 })
    }
  }

  /**
   * 获取图片属性
   * @param file
   */
  @action
  getImageProps(file: File) {
    const reader = new FileReader()
    // eslint-disable-next-line consistent-this
    reader.onload = (e: ProgressEvent<FileReader>) => {
      const img = new Image()
      img.onload = () => {
        // 获取图片的宽和高
        const width = img.width
        const height = img.height
        this.imageProps = {
          width,
          height
        }
      }

      img.src = e.target?.result as string
    }

    reader.readAsDataURL(file)
  }
}

export default new HomeStore()
