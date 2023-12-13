/**
 * @fileOverview 首页窗口
 * @date 2023-12-11
 * @author poohlaha
 */
import React, { ReactElement, useRef } from 'react'
import { observer } from 'mobx-react-lite'
import { useStore } from '@stores/index'
import { Button } from 'antd'
import Loading from '@views/components/loading/loading'
import Utils from '@utils/utils'
import { open } from '@tauri-apps/plugin-dialog'
import Archive from '@pages/home/archive'
import Preview from '@pages/home/preview'
import Look from '@pages/home/look'

const Home: React.FC<IRouterProps> = (props: IRouterProps): ReactElement => {
  const uploadRef = useRef(null)
  const { homeStore } = useStore()

  const isEmpty = () => {
    if (typeof homeStore.content === 'string') {
      return Utils.isBlank(homeStore.content)
    }

    return Utils.isObjectNull(homeStore.content)
  }

  const getEmptyHtml = () => {
    return (
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
            await homeStore.readFile(files[0])
          }}
          onClick={async () => {
            const file: any = await open({
              multiple: false,
              directory: false,
            })
            console.log('select file', file)
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
    )
  }

  const getLookHtml = () => {
    if (homeStore.suffixProps.type === 'archive') {
      return <Archive />
    }

    if (homeStore.suffixProps.type === 'preview') {
      return (
        <Preview
          fileName={homeStore.fileName || ''}
          content={homeStore.content || []}
          loading={homeStore.loading}
          suffixProps={homeStore.suffixProps || []}
        />
      )
    }

    return (
      <Look
        fileName={homeStore.fileName || ''}
        content={homeStore.content || ''}
        loading={homeStore.loading}
        suffixProps={homeStore.suffixProps || []}
      />
    )
  }

  const render = () => {
    let empty = isEmpty()
    return (
      <div className="page home-page flex-direction-column wh100">
        {!Utils.isBlank(homeStore.fileName) && (
          <div className="file-content flex-align-center w100%">
            <div className="file flex-align-center flex-1 overflow-hidden">
              <p className="text">文件名:</p>
              <p className="name flex-1 over-ellipsis overflow-hidden">{homeStore.fileName || ''}</p>
            </div>

            <div className="file-right">
              {homeStore.suffixProps?.type === 'archive' && (
                <Button type="primary" onClick={async () => homeStore.unarchive()}>
                  解压
                </Button>
              )}
              <Button className="reset" type="link" onClick={() => homeStore.reset()}>
                重置
              </Button>
            </div>
          </div>
        )}

        <div
          className={`content-box flex-1 wh100 ${empty ? 'is-empty' : ''} ${
            homeStore.suffixProps.type === 'preview' ? '' : 'wrapper overflow-auto '
          }`}
        >
          {empty ? getEmptyHtml() : getLookHtml()}
        </div>

        {/* */}
        <Loading show={homeStore.loading} />
      </div>
    )
  }

  return render()
}

export default observer(Home)
