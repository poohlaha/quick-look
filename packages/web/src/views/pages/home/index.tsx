
import React, {ReactElement, useRef} from 'react'
import { observer } from 'mobx-react-lite'
import { useStore } from '@stores/index'
import { Button } from 'antd'
import Loading from '@views/components/loading/loading'
import Utils from '@utils/utils'
import { open } from '@tauri-apps/plugin-dialog'
import Archive from '@pages/home/archive'

const Home: React.FC<IRouterProps> = (props: IRouterProps): ReactElement => {

  const uploadRef = useRef(null)
  const {homeStore} = useStore()

  const getFileSuffix = () => {
    if (Utils.isBlank(homeStore.fileName)) return 'txt'

    let suffixList = homeStore.fileName.split('.')
    if (suffixList.length === 0) {
      return homeStore.fileName
    }

    let suffix = suffixList[suffixList.length - 1]

    if (!Utils.isBlank(suffix)) {
      return suffix
    }

    // 没有 suffix, 取文件名小写
    return suffixList[0] || 'txt'
  }

  const getLookHtml = () => {
    if (typeof homeStore.content === 'object') {
      return <Archive />
    }

    return getTextOrImageHtml()
  }

  const getTextOrImageHtml = () => {
    if (typeof homeStore.content !== 'string') return null
    if (homeStore.loading || Utils.isBlank(homeStore.content) || Utils.isBlank(homeStore.fileName)) return null

    // @ts-ignore
    let prism = window['Prism']
    let suffix = getFileSuffix()

    // image
    let suffixProps: {[K: string]: any} = homeStore.suffixProps || {}
    if (suffixProps.type === 'image') {
      return (
          <div className="image-wrapper">
            <img src={homeStore.content || ''} className="wh100" />
          </div>
      )
    }

    let language = prism.languages[suffix]

    const html = prism.highlight(homeStore.content, language, suffix)
    return (
        <pre>
        <code className={`file-detail language-${suffix}`} dangerouslySetInnerHTML={{ __html: html || '' }} />
      </pre>
    )
  }

  const isEmpty = () => {
    if (typeof homeStore.content === "string") {
      return Utils.isBlank(homeStore.content)
    }

    return Utils.isObjectNull(homeStore.content)
  }

  const render = () => {
    let empty = isEmpty()
    return (
      <div className="page home-page flex-direction-column wh100">
        {
          !Utils.isBlank(homeStore.fileName) && (
                <div className="file-content flex-align-center">
                  <div className="file flex-align-center flex-1">
                    <p className="text">文件名:</p>
                    <p className="name">{homeStore.fileName || ''}</p>
                  </div>

                  <div className="file-right">
                    <Button type="link" onClick={() => homeStore.reset()}>重置</Button>
                  </div>
                </div>
            )
        }

        <div className={`wrapper flex-1 overflow-auto ${empty ? 'is-empty' : ''}`}>
          {
            empty ? (
                <div className="upload-wrapper wh100">
                  {/*
                  <Dragger {...uploadProps}>
                    <p className="ant-upload-drag-icon">
                    </p>
                    <p className="ant-upload-text">拖拽文件或点此处打开文件</p>
                    <p className="ant-upload-hint">
                      支持 .js/.css/.ts/.zip 等格式
                    </p>
                  </Dragger>
                  */}

                  <div
                      className="ant-upload-wrapper"
                      ref={uploadRef}
                      onDragOver={(event: any) => {
                        event.preventDefault()
                        // @ts-ignore
                        uploadRef.current?.classList.add('drag')
                      }}
                      onDragLeave={(event: any) => {
                        // @ts-ignore
                        uploadRef.current?.classList.remove('drag')
                      }}
                      onDrop={async (event: any) => {
                        event.preventDefault()
                        // @ts-ignore
                        uploadRef.current?.classList.remove('drag')

                        let files = event.dataTransfer?.files || []
                        if (files.length === 0) return
                        console.log(files)
                        await homeStore.readFile(files[0])
                      }}
                      onClick={async () => {
                        const file: any = await open({
                          multiple: false,
                          directory: false
                        });
                        console.log('select file', file);
                        if (file) {
                          await homeStore.readFile(null, file || {})
                        }
                      }}
                  >
                    <div className="css-dev-only-do-not-override-xu9wm8 ant-upload ant-upload-drag">
                      <span className="ant-upload ant-upload-btn" role="button">
                        <div className="ant-upload-drag-container">
                          <p className="ant-upload-drag-icon"></p>
                          <p className="ant-upload-text">拖拽文件或点击打开文件</p>
                          <p className="ant-upload-hint">支持 .js/.css/.ts/.zip 等格式</p>
                        </div>
                      </span>
                    </div>
                  </div>
                </div>
            ) : (
                getLookHtml()
            )
          }
        </div>

        {/* */}
        <Loading show={homeStore.loading}/>
      </div>
    )
  }

  return render()
}

export default observer(Home)
