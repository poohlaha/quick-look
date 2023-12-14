/**
 * @fileOverview pdf、doc、ppt查看
 * @date 2023-12-12
 * @author poohlaha
 */
import React, {ReactElement, useRef, useState} from 'react'
import { observer } from 'mobx-react-lite'
import Loading from '@views/components/loading/loading'
import Utils from '@utils/utils'
import { TOAST } from '@utils/base'
import { message } from 'antd'
import useMount from '@hooks/useMount'

interface IPreviewProps {
  fileName: string
  content: Array<{ [K: string]: any }>
  loading: boolean
  suffixProps: { [K: string]: any }
}

const Preview: React.FC<IPreviewProps> = (props: IPreviewProps): ReactElement => {
  const scaleDefault: number = 1
  const [inputFocus, setInputFocus] = useState(false)
  const [currentPage, setCurrentPage] = useState('1')
  const [oldCurrentPage, setOldCurrentPage] = useState('1')
  const [pageNumber, setPageNumber] = useState(1)
  const [scale, setScale] = useState(scaleDefault)
  const scrollRef = useRef(null)

  const SCALE_STEP: number = 0.2
  const SCALE_MIN: number = 0.4 // 倍数
  const SCALE_MAX: number = 4 // 倍数

  const scrollTo = (value: number) => {
    let imageDom = document.getElementById(`image-${value || 1}`)
    if (!imageDom) return
    imageDom.scrollIntoView({ behavior: 'smooth' })
  }

  // 判断左边是否在可视区域内
  const scrollToLeftPreview = (value: number) => {
    let leftDom = document.querySelector('.preview-left')
    if (!leftDom) {
      return
    }

    // @ts-ignore
    let leftImageDom = document.getElementById(`preview-image-${value || 1}`)
    if (!leftImageDom) return

    let images = leftDom.querySelectorAll('.image-box') || []
    let imageList: Array<{[K: string]: any}> = getImageList(images, 8)
    if (imageList.length === 0) {
      return
    }

    let image = imageList[value - 1] || {}
    if (Utils.isObjectNull(image)) {
      return
    }

    let domTop = leftDom.scrollTop
    let rect = leftDom.getBoundingClientRect() || {}

    let imageTop = image.top - domTop
    let imageBottom = image.bottom - domTop
    let top = rect.top - 44
    let bottom = rect.bottom - 44

    let isInArea = imageTop >= top && imageBottom <= bottom
    if (!isInArea) {
      leftImageDom.scrollIntoView({ behavior: 'smooth' })
    }
  }

  // 缩小
  const onScaleMin = () => {
    const min = scaleDefault * SCALE_MIN
    if (Math.fround(scale) === Math.fround(min)) {
      return
    }

    let scaleMin = scale - scaleDefault * SCALE_STEP
    if (scaleMin < min) {
      scaleMin = min
    }

    setScale(scaleMin)

    message.destroy()
    TOAST.show({
      message: `${Math.fround(scaleMin * 100)}%`,
      type: 2,
    })
  }

  useMount(() => {
    getPage()
  })

  const getImageList = (images: any = [], padding: number = 0) => {
    let imageList: Array<{[K: string]: any}> = []
    for (let i = 0; i < images.length; i++) {
      const image = images[i]
      const rect = image.getBoundingClientRect()
      let top = i * rect.height
      imageList.push({
        top: top + (i > 0 ? padding : 0),
        bottom: top + rect.height
      })
    }

    return imageList
  }


  // 防抖
  const debounce = (fn: Function, delay: number = 300) => {
    let timer: any = null
    let handler = function () {
      if (timer) {
        clearTimeout(timer)
      }

      // @ts-ignore
      let that = this
      let args = arguments
      timer = setTimeout(() => {
        fn.apply(that, args)
      }, delay)
    }

    // @ts-ignore
    handler.cancel = () => {
      if (timer) clearTimeout(timer)
    }

    return handler
  }

  const onScroll = debounce((dom: HTMLDivElement, imageList: Array<{[K: string]: any}>) => {
    console.log('scroll')
    let domTop = dom.scrollTop
    for (let i = 0; i < imageList.length; i++) {
      const image: {[K: string]: any} = imageList[i] || {}
      if (domTop >= image.top && domTop <= image.bottom) {
        setCurrentPage(`${i + 1}`)
        setOldCurrentPage(`${i + 1}`)
        setPageNumber(i + 1 || 1)
        scrollToLeftPreview(i + 1)
        break
      }
    }
  }, 50)

  const getPage = () => {
    if (!scrollRef || !scrollRef.current) return

    // @ts-ignore
    let dom = scrollRef.current as HTMLDivElement
    let images = dom.querySelectorAll('.image-box') || []
    let imageList: Array<{[K: string]: any}> = getImageList(images)

    dom.addEventListener('scroll', onScroll.bind(dom, dom, imageList))
  }

  // 放大
  const onScaleMax = () => {
    const max = scaleDefault * SCALE_MAX
    if (Math.fround(scale) === Math.fround(max)) {
      return
    }
    let scaleMax = scale + scaleDefault * SCALE_STEP

    if (scaleMax > max) {
      scaleMax = max
    }

    setScale(scaleMax)

    message.destroy()
    TOAST.show({
      message: `${Math.fround(scaleMax * 100)}%`,
      type: 2,
    })
  }

  const render = () => {
    if (!Array.isArray(props.content)) return <div></div>
    let content = props.content || []
    if (content.length === 0) return <div></div>

    return (
      <div className="preview wh100">
        <div className="preview-wrapper flex wh100">
          <div className="preview-left h100 overflow-y-auto flex-direction-column flex-align-center">
            {content.map((item: { [K: string]: any } = {}, index: number) => {
              return (
                <div
                    id={`preview-image-${index + 1}`}
                    className={`image-box flex-direction-column ${pageNumber === index + 1 ? 'active' : ''}`}
                    key={index}
                    onClick={() => {
                      setCurrentPage(`${index + 1}`)
                      setOldCurrentPage(`${index + 1}`)
                      setPageNumber(index + 1)
                      scrollTo(index + 1)
                    }}
                >
                  <img src={item.content || ''} className="w100 flex-1" />
                  <p className="text flex-center">{index + 1}</p>
                </div>
              )
            })}
          </div>

          <div className="preview-right h100 overflow-auto flex-1 flex-direction-column flex-align-center" ref={scrollRef}>
            {content.map((item: { [K: string]: any } = {}, index: number) => {
              return (
                <div className="image-box" key={index} id={`image-${index + 1}`}>
                  <img src={item.content || ''} style={{ transform: `scale(${scale})` }} />
                </div>
              )
            })}
          </div>

          <div className="actions-box preview-actions">
            <div className="actions">
              {/* 缩小 */}
              <div className="operation" onClick={onScaleMin}>
                <span className="action">
                  <svg
                    className="svg-icon"
                    viewBox="64 64 896 896"
                    focusable="false"
                    data-icon="zoom-out"
                    fill="currentColor"
                    aria-hidden="true"
                  >
                    <path d="M637 443H325c-4.4 0-8 3.6-8 8v60c0 4.4 3.6 8 8 8h312c4.4 0 8-3.6 8-8v-60c0-4.4-3.6-8-8-8zm284 424L775 721c122.1-148.9 113.6-369.5-26-509-148-148.1-388.4-148.1-537 0-148.1 148.6-148.1 389 0 537 139.5 139.6 360.1 148.1 509 26l146 146c3.2 2.8 8.3 2.8 11 0l43-43c2.8-2.7 2.8-7.8 0-11zM696 696c-118.8 118.7-311.2 118.7-430 0-118.7-118.8-118.7-311.2 0-430 118.8-118.7 311.2-118.7 430 0 118.7 118.8 118.7 311.2 0 430z"></path>
                  </svg>
                </span>
              </div>

              {/* 放大 */}
              <div className="operation" onClick={onScaleMax}>
                <span className="action">
                  <svg
                    className="svg-icon"
                    viewBox="64 64 896 896"
                    focusable="false"
                    data-icon="zoom-in"
                    fill="currentColor"
                    aria-hidden="true"
                  >
                    <path d="M637 443H519V309c0-4.4-3.6-8-8-8h-60c-4.4 0-8 3.6-8 8v134H325c-4.4 0-8 3.6-8 8v60c0 4.4 3.6 8 8 8h118v134c0 4.4 3.6 8 8 8h60c4.4 0 8-3.6 8-8V519h118c4.4 0 8-3.6 8-8v-60c0-4.4-3.6-8-8-8zm284 424L775 721c122.1-148.9 113.6-369.5-26-509-148-148.1-388.4-148.1-537 0-148.1 148.6-148.1 389 0 537 139.5 139.6 360.1 148.1 509 26l146 146c3.2 2.8 8.3 2.8 11 0l43-43c2.8-2.7 2.8-7.8 0-11zM696 696c-118.8 118.7-311.2 118.7-430 0-118.7-118.8-118.7-311.2 0-430 118.8-118.7 311.2-118.7 430 0 118.7 118.8 118.7 311.2 0 430z"></path>
                  </svg>
                </span>
              </div>

              {/* 左边 */}
              <div
                className={`operation action-left ${pageNumber === 1 ? 'disabled' : ''}`}
                onClick={() => {
                  if (pageNumber === 1) return
                  let page = pageNumber - 1
                  if (page <= 1) {
                    page = 1
                  }

                  setCurrentPage(`${page}`)
                  setOldCurrentPage(`${page}`)
                  setPageNumber(page)
                  scrollTo(page)
                  scrollToLeftPreview(page)
                }}
              >
                <span className="action">
                  <svg className="svg-icon" version="1.1" viewBox="0 0 16 16">
                    <defs>
                      <clipPath>
                        <rect x="0" y="0" width="16" height="16" rx="0" />
                      </clipPath>
                    </defs>
                    <g clipPath="url(#master_svg0_219_610/335_04764)">
                      <g>
                        <path
                          d="M11.04022609375,4.707482L11.04060609375,4.707107Q11.181256093750001,4.566455,11.25737609375,4.382683Q11.33349609375,4.198912,11.33349609375,4Q11.33349609375,3.801088,11.25737609375,3.617317Q11.181256093750001,3.433545,11.04060609375,3.292893L11.04002609375,3.292322Q11.03602609375,3.28832,11.03196609375,3.284365L11.03139609375,3.2838000000000003Q10.891336093749999,3.147326,10.71019609375,3.073663Q10.52904609375,3,10.33349609375,3L10.32796609375,3.000015Q10.13049609375,3.001107,9.94825609375,3.077182Q9.76602609375,3.153257,9.62638609375,3.292893L5.62638909375,7.29289Q5.48573709375,7.43355,5.4096160937499995,7.617319999999999Q5.33349609375,7.80109,5.33349609375,8Q5.33349609375,8.19891,5.4096160937499995,8.38268Q5.48573709375,8.56645,5.62638909375,8.70711L9.62638609375,12.70711Q9.76704609375,12.84776,9.95081609375,12.92388Q10.13458609375,13,10.33349609375,13Q10.35938609375,13,10.385236093749999,12.99866Q10.570486093749999,12.98906,10.739966093749999,12.91367Q10.90943609375,12.83827,11.04060609375,12.70711Q11.181256093750001,12.56645,11.25737609375,12.38268Q11.33349609375,12.19891,11.33349609375,12Q11.33349609375,11.80109,11.25737609375,11.61732Q11.181256093750001,11.43355,11.04060609375,11.29289L11.04008609375,11.29238L7.74770609375,8L11.04022609375,4.707482Z"
                          fillRule="evenodd"
                          fill="#000000"
                          fillOpacity="0.6499999761581421"
                        />
                      </g>
                    </g>
                  </svg>
                </span>
              </div>

              {/* 输入页数 */}
              <div className={`current-page ${inputFocus ? 'focus' : ''}`}>
                <input
                    className="wh100"
                    value={currentPage}
                    onFocus={e => {
                      e.target.select()
                      setInputFocus(true)
                    }}
                    onBlur={() => {
                      setInputFocus(false)
                      if (Utils.isBlank(currentPage)) {
                        setCurrentPage(oldCurrentPage)
                      }
                    }}
                    onChange={(e: any) => {
                      let value = e.target.value || ''
                      setCurrentPage(value)
                      if (!Utils.isBlank(value)) {
                        let valueInt = parseInt(value)
                        if (valueInt <= 0) {
                          valueInt = 1
                        }
                        if (valueInt > content.length) {
                          valueInt = content.length
                        }
                        setCurrentPage(`${valueInt}`)
                        setOldCurrentPage(`${valueInt}`)
                        setPageNumber(valueInt || 1)
                        scrollTo(valueInt)
                      }
                    }}
                />
              </div>

              <div className="spec">/</div>

              {/* 总页数 */}
              <div className="total-page flex-center">{content.length || 0}</div>

              {/* 右边 */}
              <div
                className={`operation action-right ${pageNumber === content.length ? 'disabled' : ''}`}
                onClick={() => {
                  if (pageNumber === content.length) return
                  let page = pageNumber + 1
                  if (page >= content.length) {
                    page = content.length
                  }

                  setCurrentPage(`${page}`)
                  setOldCurrentPage(`${page}`)
                  setPageNumber(page)
                  scrollTo(page)
                  scrollToLeftPreview(page)
                }}
              >
                <span className="action">
                  <svg className="svg-icon" version="1.1" viewBox="0 0 16 16">
                    <defs>
                      <clipPath>
                        <rect x="0" y="0" width="16" height="16" rx="0" />
                      </clipPath>
                    </defs>
                    <g clipPath="url(#master_svg0_219_610/335_04764)">
                      <g>
                        <path
                          d="M11.04022609375,4.707482L11.04060609375,4.707107Q11.181256093750001,4.566455,11.25737609375,4.382683Q11.33349609375,4.198912,11.33349609375,4Q11.33349609375,3.801088,11.25737609375,3.617317Q11.181256093750001,3.433545,11.04060609375,3.292893L11.04002609375,3.292322Q11.03602609375,3.28832,11.03196609375,3.284365L11.03139609375,3.2838000000000003Q10.891336093749999,3.147326,10.71019609375,3.073663Q10.52904609375,3,10.33349609375,3L10.32796609375,3.000015Q10.13049609375,3.001107,9.94825609375,3.077182Q9.76602609375,3.153257,9.62638609375,3.292893L5.62638909375,7.29289Q5.48573709375,7.43355,5.4096160937499995,7.617319999999999Q5.33349609375,7.80109,5.33349609375,8Q5.33349609375,8.19891,5.4096160937499995,8.38268Q5.48573709375,8.56645,5.62638909375,8.70711L9.62638609375,12.70711Q9.76704609375,12.84776,9.95081609375,12.92388Q10.13458609375,13,10.33349609375,13Q10.35938609375,13,10.385236093749999,12.99866Q10.570486093749999,12.98906,10.739966093749999,12.91367Q10.90943609375,12.83827,11.04060609375,12.70711Q11.181256093750001,12.56645,11.25737609375,12.38268Q11.33349609375,12.19891,11.33349609375,12Q11.33349609375,11.80109,11.25737609375,11.61732Q11.181256093750001,11.43355,11.04060609375,11.29289L11.04008609375,11.29238L7.74770609375,8L11.04022609375,4.707482Z"
                          fillRule="evenodd"
                          fill="#000000"
                          fillOpacity="0.6499999761581421"
                        />
                      </g>
                    </g>
                  </svg>
                </span>
              </div>
            </div>
          </div>
        </div>

        <Loading show={props.loading} />
      </div>
    )
  }

  return render()
}

export default observer(Preview)
