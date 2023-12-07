/**
 * @fileOverview dashboard store
 * @date 2023-07-05
 * @author poohlaha
 */
import { observable, action } from 'mobx'
import BaseStore from '../base/base.store'
import { invoke } from '@tauri-apps/api/primitives'

class HomeStore extends BaseStore {
  @observable fileName = '' // 文件名称
  @observable content = '' // 文件内容
  @observable imageSuffixes: Array<string> = [] // 图片后续列表


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
    this.loading = true
    this.fileName = file.name || ''
    let buffer: ArrayBuffer = await file.arrayBuffer()

    let result: {[K: string]: any} = await invoke('open_file', buffer, { headers: { fileName: this.fileName }})
    console.log('result:', result)
    this.loading = false

    this.content = this.analysisResult(result, `读取文件 ${this.fileName} 失败!`)
    this.imageSuffixes = (result.imageSuffixes || '').split(',') || []

    console.log('content:', this.content)
  }
}

export default new HomeStore()
