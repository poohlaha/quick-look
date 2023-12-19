/**
 * @fileOverview 查看文件
 * @date 2023-12-11
 * @author poohlaha
 */
import React, { ReactElement } from 'react'
import { observer } from 'mobx-react-lite'
import Utils from '@utils/utils'
import Image from './image'
import CodeMirror from '@uiw/react-codemirror'
import { langs, langNames } from '@uiw/codemirror-extensions-langs'
import { githubLight } from '@uiw/codemirror-theme-github'

interface ILookProps {
  fileName: string
  content: string | { [K: string]: any } | Array<any>
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
    let content = ''
    if (Array.isArray(props.content)) {
      content = props.content.join('\n')
    } else if (typeof props.content !== 'string') {
      content = JSON.stringify(props.content)
    } else {
      content = props.content
    }

    if (props.loading || Utils.isBlank(content) || Utils.isBlank(props.fileName)) return <div></div>

    // @ts-ignore
    let prism = window['Prism']
    let suffix = getFileSuffix()

    // image
    let suffixProps: { [K: string]: any } = props.suffixProps || {}
    if (suffixProps.type === 'image') {
      return <Image content={content || ''} type={suffixProps.name || ''} />
    }

    if (suffix === 'plist') {
      suffix = 'xml'
    } else if (suffix === 'rs') {
      suffix = 'rust'
    } else if (suffix === 'js') {
      suffix = 'javascript'
    }

    /*
   let language
   try {
     language = prism.languages[suffix] || prism.languages['txt']
   } catch (e) {
     console.warn('no language has found, use default `txt`')
     language = prism.languages['txt']
   }

   const html = prism.highlight(content, language, suffix)

   return (
     <pre>
       <code className={`file-detail language-${suffix}`} dangerouslySetInnerHTML={{ __html: html || '' }} />
     </pre>
   )

    */
    console.log(langNames)
    // @ts-ignore
    let extension = langs[suffix]
    if (!extension) {
      // @ts-ignore
      extension = langs['apl']
    }

    return <CodeMirror value={content} theme={githubLight} extensions={[extension?.()]} />
  }

  return render()
}

export default observer(Look)
