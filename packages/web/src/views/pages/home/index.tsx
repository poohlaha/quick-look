
import React, {ReactElement} from 'react'
import { observer } from 'mobx-react-lite'
import { useStore } from '@stores/index'
import type { UploadProps } from 'antd'
import { Upload, Button } from 'antd'
import Loading from '@views/components/loading/loading'
import Utils from '@utils/utils'

const { Dragger } = Upload

const Home: React.FC<IRouterProps> = (props: IRouterProps): ReactElement => {

  const {homeStore} = useStore()

  const uploadProps: UploadProps = {
    name: 'file',
    multiple: true,
    action: async (file) => {
      console.log('file', file)
      await homeStore.readFile(file)
    },
    onDrop(e) {
      console.log('Dropped files', e.dataTransfer.files)
    }
  }

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
    if (homeStore.loading || Utils.isBlank(homeStore.content) || Utils.isBlank(homeStore.fileName)) return null

    // @ts-ignore
    let prism = window['Prism']
    let suffix = getFileSuffix()

    // image
    if (homeStore.imageSuffixes.includes(suffix)) {
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

  const render = () => {
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

        <div className="wrapper flex-center flex-1 overflow-auto">
          {
            Utils.isBlank(homeStore.content) ? (
                <Dragger {...uploadProps}>
                  <p className="ant-upload-drag-icon">
                  </p>
                  <p className="ant-upload-text">拖拽文件或点此处打开文件</p>
                  <p className="ant-upload-hint">
                    支持 .js/.css/.ts/.zip 等格式
                  </p>
                </Dragger>
            ) : (
                getLookHtml()
            )
          }
        </div>

        {/* */}
        <Loading show={homeStore.loading} />
      </div>
    )
  }

  return render()
}

export default observer(Home)
