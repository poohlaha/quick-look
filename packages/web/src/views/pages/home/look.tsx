/**
 * @fileOverview 查看文件
 * @date 2023-12-11
 * @author poohlaha
 */
import React, { ReactElement } from 'react'
import { observer } from 'mobx-react-lite'
import Utils from '@utils/utils'
import Image from './image'

interface ILookProps {
  fileName: string
  content: string | { [K: string]: any }
  loading: boolean
  suffixProps: { [K: string]: any }
}

const Look: React.FC<ILookProps> = (props: ILookProps): ReactElement => {
  const getFileSuffix = () => {
    if (Utils.isBlank(props.fileName)) return 'txt'

    let suffixList = props.fileName.split('.')
    if (suffixList.length === 0) {
      return props.fileName
    }

    let suffix = suffixList[suffixList.length - 1]

    if (!Utils.isBlank(suffix)) {
      return suffix
    }

    // 没有 suffix, 取文件名小写
    return suffixList[0] || 'txt'
  }

  const render = () => {
    if (typeof props.content !== 'string') return <div></div>
    if (props.loading || Utils.isBlank(props.content) || Utils.isBlank(props.fileName)) return <div></div>

    // @ts-ignore
    let prism = window['Prism']
    let suffix = getFileSuffix()

    // image
    let suffixProps: { [K: string]: any } = props.suffixProps || {}
    if (suffixProps.type === 'image') {
      return <Image content={props.content || ''} />
    }

    if (suffix === 'plist') {
      suffix = 'xml'
    } else if (suffix === 'rs') {
      suffix = 'rust'
    }

    let language
    try {
      language = prism.languages[suffix] || prism.languages['txt']
    } catch (e) {
      console.warn('no language has found, use default `txt`')
      language = prism.languages['txt']
    }

    const html = prism.highlight(props.content, language, suffix)
    return (
      <pre>
        <code className={`file-detail language-${suffix}`} dangerouslySetInnerHTML={{ __html: html || '' }} />
      </pre>
    )
  }

  return render()
}

export default observer(Look)
