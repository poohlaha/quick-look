/**
 * @fileOverview dashboard store
 * @date 2023-07-05
 * @author poohlaha
 */
import { observable, action } from 'mobx'
import BaseStore from '../base/base.store'
import { invoke } from '@tauri-apps/api/primitives'
import { info } from '@tauri-apps/plugin-log'
import { COMMON, TOAST } from '@utils/base'
import Utils from '@utils/utils'

class HomeStore extends BaseStore {
  @observable fileName = '' // 文件名称
  @observable content: any = {} // 文件内容
  @observable suffixProps: { [K: string]: any } = {} // 图片后续列表
  @observable imageProps: { [K: string]: number | string } = {} // 图片属性
  @observable fileProps: { [K: string]: any } = {} // 文件属性
  @observable detailContent = {
    data: '',
    fileName: '',
  }

  @observable detailLoading: boolean = false

  @action
  reset() {
    this.fileName = ''
    this.content = ''
    this.loading = false
  }

  async readDetailFile(path: string, name: string, callback?: Function) {
    try {
      if (Utils.isBlank(path) || Utils.isBlank(name)) return
      this.detailContent = {
        data: '',
        fileName: '',
      }

      this.detailLoading = true
      this.detailContent.fileName = name || ''

      let result: { [K: string]: any } = await invoke(
        'file_handler',
        { filePath: path },
        { headers: { fileName: encodeURIComponent(this.detailContent.fileName) } }
      )
      this.detailLoading = false

      if (Utils.isBlank(result.body || '')) {
        TOAST.show({ message: '当前文件不支持查看 !', type: 3 })
        return
      }

      this.suffixProps = result.suffixProps || []
      await info(`readDetailFile suffixProps: ${JSON.stringify(this.suffixProps)}`)
      console.log('readDetailFile suffixProps:', this.suffixProps)

      this.detailContent.data = this.analysisResult(result, `读取文件 ${name || ''} 失败!`)
      callback?.()
    } catch (err: any) {
      this.detailLoading = false
      this.detailContent = {
        data: '',
        fileName: '',
      }
      console.error('read file error !', err)
      TOAST.show({ message: `读取文件 ${name || ''} 失败!`, type: 4 })
    }
  }

  /**
   * 获取系统信息
   */
  @action
  async readFile(file: File | null = null, fileProps: { [K: string]: string } = {}) {
    try {
      this.imageProps = {}
      this.loading = true
      let result: { [K: string]: any } = {}
      this.content = ''

      if (file !== null) {
        this.fileName = file.name || ''
        let buffer: ArrayBuffer = await file.arrayBuffer()
        result = await invoke('file_handler', buffer, { headers: { fileName: encodeURIComponent(this.fileName) } })
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

        result = await invoke(
          'file_handler',
          { filePath },
          { headers: { fileName: encodeURIComponent(this.fileName) } }
        )
      }

      console.log('result:', result)
      this.loading = false

      this.fileProps = result.fileProps || {}
      this.suffixProps = result.suffixProps || {}
      await info(`suffixProps: ${JSON.stringify(this.suffixProps)}`)
      console.log('suffixProps:', this.suffixProps)

      this.content = this.analysisResult(result, `读取文件 ${this.fileName} 失败!`)
      if (this.content === undefined || this.content === null) {
        this.reset()
        return
      }

      if (typeof this.content === 'object') {
        let files = this.content.files || []
        this.changeFiles(files)
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
          height,
        }
      }

      img.src = e.target?.result as string
    }

    reader.readAsDataURL(file)
  }

  @action
  changeFiles(files: Array<{ [K: string]: any }> = []) {
    files.forEach(file => {
      if (file.files && Array.isArray(file.files)) {
        this.changeFiles(file.files)
        file.children = file.files
        delete file.files
        if (file.children.length === 0) {
          delete file.children
        }
      }
    })
  }

  /**
   * 解压
   */
  @action
  async unarchive() {
    console.log(this.fileProps)
    if (Utils.isBlank(this.fileProps.path) || Utils.isBlank(this.fileProps.fullPath || '')) {
      TOAST.show({ message: '解压失败, 路径不存在!', type: 4 })
      return
    }

    this.loading = true
    let result: { [K: string]: any } = await invoke('unarchive', {
      filePath: this.fileProps.path,
      fullPath: this.fileProps.fullPath || '',
    })
    this.loading = false

    console.log('result: ', result)

    if (!Utils.isBlank(result.error || '') || result.code !== 200) {
      TOAST.show({ message: result.error || `解压 ${this.fileProps.name || ''} 失败`, type: 4 })
      return
    }

    TOAST.show({ message: `解压 ${this.fileProps.name || ''} 成功`, type: 2 })
  }
}

export default new HomeStore()
