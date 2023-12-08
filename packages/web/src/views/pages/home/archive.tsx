import React, {ReactElement} from "react";
import {observer} from "mobx-react-lite";
import {useStore} from '@views/stores'
import Utils from '@utils/utils'
import { Table } from 'antd'

const Home: React.FC<IRouterProps> = (props: IRouterProps): ReactElement => {

    const {homeStore} = useStore()

    const columns: any = [
        {
            title: 'Name',
            dataIndex: 'name',
            key: 'name',
        },
        {
            title: 'Date Modified',
            dataIndex: 'modified',
            key: 'modified',
        },
        {
            title: 'Size',
            dataIndex: 'size',
            key: 'size',
        },
        {
            title: 'Packed',
            dataIndex: 'packed',
            key: 'packed',
        },
        {
            title: 'Kind',
            dataIndex: 'kind',
            key: 'kind',
        }
    ]

    const render = () => {
        if (typeof homeStore.content !== "object") return (<div></div>)
        if (homeStore.loading || Utils.isObjectNull(homeStore.content) || Utils.isBlank(homeStore.fileName)) return (<div></div>)

        return (
            <div className="archive-wrapper">
                <div className="info flex">
                    <div className="svg-box">
                        <svg className="svg-icon wh100" viewBox="0 0 1024 1024" version="1.1" xmlns="http://www.w3.org/2000/svg">
                            <path d="M0 339.53792h1022.0032v344.92416H0z" fill="#F9605D"></path>
                            <path d="M1016.5248 339.53792V64.73728c0-34.11968-29.19424-64.44032-63.86688-64.44032H63.87712C29.20448 0.29696 0 30.6176 0 64.73728v274.80064h1016.5248z" fill="#53C7F7"></path>
                            <path d="M0 684.46208v274.80064c0 34.11968 29.20448 64.44032 63.87712 64.44032h896.07168c34.68288 0 63.87712-30.32064 63.87712-64.44032V684.46208H0z" fill="#7DCF3B"></path>
                            <path d="M399.6672 0.29696H627.8144V1023.6928H399.6672z" fill="#FDB042"></path>
                            <path d="M627.80416 443.77088v144.03584H388.73088V443.77088h239.07328z m38.32832-58.74688H350.40256c-5.4784 0-14.60224 5.6832-14.60224 15.1552v229.3248c0 5.6832 5.4784 15.1552 14.60224 15.1552h315.72992c5.46816 0 14.592-5.6832 14.592-15.1552V400.1792c-3.64544-11.3664-9.1136-15.1552-14.592-15.1552z" fill="#FFFFFF"></path>
                        </svg>
                    </div>

                    <div className="descriptions">
                        <p className="title">{homeStore.fileName || ''}</p>
                        <div className="desc">
                            <span>Path:</span>
                            <span>{homeStore.content.path || ''}</span>
                        </div>
                        <div className="desc">
                            <span>Kind:</span>
                            <span>{homeStore.content.kind || ''}</span>
                        </div>
                        <div className="desc">
                            <span>Date Modified:</span>
                            <span>{homeStore.content.modified || ''}</span>
                        </div>
                        <div className="desc">
                            <span>Size:</span>
                            <span>{homeStore.content.size || ''}</span>
                        </div>
                        <div className="desc">
                            <span>Packed:</span>
                            <span>{homeStore.content.packed || ''}</span>
                        </div>
                    </div>
                </div>
                <div className="list">
                    <Table
                        columns={columns}
                        dataSource={homeStore.content.files || []}
                        pagination={
                            {
                                showSizeChanger: false
                            }
                        }
                    />
                </div>
            </div>
        )
    }

    return render()
}

export default observer(Home)