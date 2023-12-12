/**
 * @fileOverview pdf、doc、ppt查看
 * @date 2023-12-12
 * @author poohlaha
 */
import React, { ReactElement } from 'react'
import { observer } from 'mobx-react-lite'
import Loading from '@views/components/loading/loading'

interface IPreviewProps {
    fileName: string
    content: Array<{[K: string]: any}>
    loading: boolean
    suffixProps: { [K: string]: any }
}

const Preview: React.FC<IPreviewProps> = (props: IPreviewProps): ReactElement => {

    const render = () => {
        if (!Array.isArray(props.content)) return (<div></div>)
        let content = props.content || []
        if (content.length === 0) return (<div></div>)

        return (
            <div className="preview">
               <div className="preview-wrapper flex">
                   <div className="preview-left h100 overflow-y-auto flex-direction-column flex-align-center">
                       {
                           content.map((item: {[K: string]: any} = {}, index: number) => {
                               return (
                                   <div className="image-box flex-direction-column" key={index}>
                                       <img src={item.content || ''} className="w100 flex-1" />
                                       <p className="text flex-center">{index + 1}</p>
                                   </div>
                               )
                           })
                       }
                   </div>

                   <div className="preview-right h100 overflow-y-auto flex-1 flex-direction-column flex-align-center">
                       {
                           content.map((item: {[K: string]: any} = {}, index: number) => {
                               return (
                                   <div className="image-box" key={index}>
                                       <img src={item.content || ''} />
                                   </div>
                               )
                           })
                       }
                   </div>
               </div>

                <Loading show={props.loading} />
            </div>
        )
    }

    return render()
}

export default observer(Preview)
