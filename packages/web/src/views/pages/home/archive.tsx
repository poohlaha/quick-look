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
            render: (text: string, record: {[K: string]: any} = {}) => {
                console.log("record", record)
                return (
                   <div className="name-cell flex-align-center">
                       {
                           record.isDirectory ? (
                               <div className="svg-box dir">
                                   <svg className="svg-icon wh100" viewBox="0 0 1024 1024" version="1.1"
                                        xmlns="http://www.w3.org/2000/svg">
                                       <path d="M848.8576 199.1936H415.7568c0-26.5728-21.5424-48.128-48.128-48.128H175.1424c-26.5728 0-48.128 21.5424-48.128 48.128V343.5648c0 26.5984 21.5424 48.1408 48.128 48.1408h673.728c26.5728 0 48.128-21.5424 48.128-48.1408v-96.2432c-0.0128-26.5856-21.5552-48.128-48.1408-48.128z" fill="#CCA352"></path>
                                       <path d="M800.7424 247.3088H223.2576c-26.5728 0-48.128 21.5424-48.128 48.128v48.128c0 26.5984 21.5424 48.1408 48.128 48.1408h577.472c26.5728 0 48.128-21.5424 48.128-48.1408v-48.128c0-26.5728-21.5424-48.128-48.1152-48.128z" fill="#FFFFFF"></path>
                                       <path d="M848.8576 295.4368H175.1424c-26.5728 0-48.128 21.5424-48.128 48.128v481.2544c0 26.5472 21.5424 48.128 48.128 48.128h673.728c26.5728 0 48.128-21.568 48.128-48.128V343.552c-0.0128-26.5728-21.5552-48.1152-48.1408-48.1152z" fill="#FFCC66"></path>
                                   </svg>
                               </div>
                           ) : (
                               <div className="svg-box file">
                                   <svg className="svg-icon" viewBox="0 0 1024 1024" version="1.1"
                                        xmlns="http://www.w3.org/2000/svg">
                                       <path d="M766.577778 884.622222h-440.888889c-31.288889 0-56.888889-25.6-56.888889-56.888889V462.222222c0-31.288889 25.6-56.888889 56.888889-56.888889h46.933333c15.644444 0 28.444444 12.8 28.444445 28.444445s-12.8 28.444444-28.444445 28.444444h-46.933333v365.511111h439.466667V554.666667h-28.444445c-15.644444 0-28.444444-12.8-28.444444-28.444445s12.8-28.444444 28.444444-28.444444h28.444445c31.288889 0 56.888889 25.6 56.888888 56.888889v273.066666c0 32.711111-24.177778 56.888889-55.466666 56.888889z" fill="#3FA6AD"></path>
                                       <path d="M736.711111 556.088889H504.888889c-9.955556 0-18.488889-5.688889-24.177778-12.8l-49.777778-79.644445h-58.311111c-8.533333 0-17.066667-4.266667-22.755555-11.377777-5.688889-7.111111-7.111111-15.644444-5.688889-24.177778l35.555555-152.177778c2.844444-12.8 11.377778-25.6 22.755556-32.711111 11.377778-7.111111 25.6-9.955556 39.822222-5.688889L763.733333 312.888889c12.8 2.844444 25.6 11.377778 32.711111 22.755555 7.111111 11.377778 9.955556 25.6 5.688889 39.822223l-36.977777 157.866666c-2.844444 12.8-14.222222 22.755556-28.444445 22.755556z m-216.177778-56.888889h193.422223l31.288888-130.844444-310.044444-73.955556-27.022222 112.355556h38.4c9.955556 0 18.488889 5.688889 24.177778 12.8l49.777777 79.644444z" fill="#3FA6AD"></path>
                                       <path d="M736.711111 556.088889H504.888889c-9.955556 0-18.488889-5.688889-24.177778-12.8l-49.777778-79.644445h-58.311111c-15.644444 0-28.444444-12.8-28.444444-28.444444s12.8-28.444444 28.444444-28.444444h73.955556c9.955556 0 18.488889 5.688889 24.177778 12.8l49.777777 79.644444h216.177778c15.644444 0 28.444444 12.8 28.444445 28.444444s-12.8 28.444444-28.444445 28.444445z" fill="#3FA6AD"></path>
                                       <path d="M661.333333 449.422222c-2.844444 0-4.266667 0-7.111111-1.422222l-164.977778-39.822222c-15.644444-4.266667-24.177778-18.488889-21.333333-34.133334 4.266667-15.644444 18.488889-24.177778 34.133333-21.333333L668.444444 393.955556c15.644444 4.266667 24.177778 18.488889 21.333334 34.133333-2.844444 12.8-15.644444 21.333333-28.444445 21.333333z" fill="#DC4569"></path>
                                   </svg>
                               </div>
                           )
                       }
                       <p className="text">{text || ''}</p>
                   </div>
                )
            },
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
        if (homeStore.loading || Utils.isObjectNull(homeStore.content) || Utils.isBlank(homeStore.fileName)) return (
            <div></div>)

        return (
            <div className="archive-wrapper">
                <div className="info flex">
                    <div className="svg-box">
                        <svg className="svg-icon wh100" viewBox="0 0 1024 1024" version="1.1"
                             xmlns="http://www.w3.org/2000/svg">
                            <path d="M0 339.53792h1022.0032v344.92416H0z" fill="#F9605D"></path>
                            <path
                                d="M1016.5248 339.53792V64.73728c0-34.11968-29.19424-64.44032-63.86688-64.44032H63.87712C29.20448 0.29696 0 30.6176 0 64.73728v274.80064h1016.5248z"
                                fill="#53C7F7"></path>
                            <path
                                d="M0 684.46208v274.80064c0 34.11968 29.20448 64.44032 63.87712 64.44032h896.07168c34.68288 0 63.87712-30.32064 63.87712-64.44032V684.46208H0z"
                                fill="#7DCF3B"></path>
                            <path d="M399.6672 0.29696H627.8144V1023.6928H399.6672z" fill="#FDB042"></path>
                            <path
                                d="M627.80416 443.77088v144.03584H388.73088V443.77088h239.07328z m38.32832-58.74688H350.40256c-5.4784 0-14.60224 5.6832-14.60224 15.1552v229.3248c0 5.6832 5.4784 15.1552 14.60224 15.1552h315.72992c5.46816 0 14.592-5.6832 14.592-15.1552V400.1792c-3.64544-11.3664-9.1136-15.1552-14.592-15.1552z"
                                fill="#FFFFFF"></path>
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