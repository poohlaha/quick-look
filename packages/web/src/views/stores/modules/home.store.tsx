/**
 * @fileOverview dashboard store
 * @date 2023-07-05
 * @author poohlaha
 */
import { observable, action } from 'mobx'
import BaseStore from '../base/base.store'
import { invoke } from '@tauri-apps/api/primitives'
import { info } from '@tauri-apps/plugin-log'
import {COMMON, TOAST} from "@utils/base";

class HomeStore extends BaseStore {
  @observable fileName = '' // 文件名称
  @observable content = '' // 文件内容
  @observable imageSuffixes: Array<string> = [] // 图片后续列表
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
  async readFile(file: File) {
    try {
      this.imageProps = {}
      this.loading = true
      this.fileName = file.name || ''
      let buffer: ArrayBuffer = await file.arrayBuffer()

      let result: {[K: string]: any} = await invoke('open_file', buffer, { headers: { fileName: encodeURIComponent(this.fileName) }})
      console.log('result:', result)
      this.loading = false

      this.content = this.analysisResult(result, `读取文件 ${this.fileName} 失败!`)
      // this.getImageProps(file)

      this.imageSuffixes = (result.imageSuffixes || '').split(',') || []
      await info(`imageSuffixes: ${JSON.stringify(this.imageSuffixes)}`)
      console.log('imageSuffixes:', this.imageSuffixes)
    } catch (err: any) {
      this.loading = false
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
