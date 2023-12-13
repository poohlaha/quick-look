/**
 * @fileOverview 图片
 * @date 2023-12-13
 * @author poohlaha
 */
import React, { ReactElement, useState } from 'react'
import { observer } from 'mobx-react-lite'

interface IImageProps {
  needActions?: boolean
  content: string
}

const Image: React.FC<IImageProps> = (props: IImageProps): ReactElement => {
  const [scaleX, setScaleX] = useState(1)
  const [scaleY, setScaleY] = useState(1)
  const [scale, setScale] = useState(1)
  const [rotate, setRotate] = useState(0)
  const [minRotate, setMinRotate] = useState(true)
  const [maxRotate, setMaxRotate] = useState(false)

  const MIN_SCALE = 1
  const MAX_SCALE = 3
  const SCALE_STEP = 0.5
  const scaleDefault = 1

  const getActionsHtml = () => {
    let needActions = props.needActions !== false
    if (!needActions) return null

    return (
      <div className="actions-box">
        <div className="actions">
          {/* 上下翻转 */}
          <div
            className="operation flipY"
            onClick={() => {
              setScaleY(-scaleY)
            }}
          >
            <span className="action">
              <svg
                className="svg-icon"
                viewBox="64 64 896 896"
                focusable="false"
                data-icon="swap"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M847.9 592H152c-4.4 0-8 3.6-8 8v60c0 4.4 3.6 8 8 8h605.2L612.9 851c-4.1 5.2-.4 13 6.3 13h72.5c4.9 0 9.5-2.2 12.6-6.1l168.8-214.1c16.5-21 1.6-51.8-25.2-51.8zM872 356H266.8l144.3-183c4.1-5.2.4-13-6.3-13h-72.5c-4.9 0-9.5 2.2-12.6 6.1L150.9 380.2c-16.5 21-1.6 51.8 25.1 51.8h696c4.4 0 8-3.6 8-8v-60c0-4.4-3.6-8-8-8z"></path>
              </svg>
            </span>
          </div>

          {/* 左右翻转 */}
          <div
            className="operation flipX"
            onClick={() => {
              setScaleX(-scaleX)
            }}
          >
            <span className="action">
              <svg
                className="svg-icon"
                viewBox="64 64 896 896"
                focusable="false"
                data-icon="swap"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M847.9 592H152c-4.4 0-8 3.6-8 8v60c0 4.4 3.6 8 8 8h605.2L612.9 851c-4.1 5.2-.4 13 6.3 13h72.5c4.9 0 9.5-2.2 12.6-6.1l168.8-214.1c16.5-21 1.6-51.8-25.2-51.8zM872 356H266.8l144.3-183c4.1-5.2.4-13-6.3-13h-72.5c-4.9 0-9.5 2.2-12.6 6.1L150.9 380.2c-16.5 21-1.6 51.8 25.1 51.8h696c4.4 0 8-3.6 8-8v-60c0-4.4-3.6-8-8-8z"></path>
              </svg>
            </span>
          </div>

          {/* 逆时针旋转 */}
          <div
            className="operation rotate-left"
            onClick={() => {
              setRotate(rotate - 90)
            }}
          >
            <span className="action">
              <svg
                className="svg-icon"
                viewBox="64 64 896 896"
                focusable="false"
                data-icon="rotate-left"
                fill="currentColor"
                aria-hidden="true"
              >
                <defs>
                  <style></style>
                </defs>
                <path d="M672 418H144c-17.7 0-32 14.3-32 32v414c0 17.7 14.3 32 32 32h528c17.7 0 32-14.3 32-32V450c0-17.7-14.3-32-32-32zm-44 402H188V494h440v326z"></path>
                <path d="M819.3 328.5c-78.8-100.7-196-153.6-314.6-154.2l-.2-64c0-6.5-7.6-10.1-12.6-6.1l-128 101c-4 3.1-3.9 9.1 0 12.3L492 318.6c5.1 4 12.7.4 12.6-6.1v-63.9c12.9.1 25.9.9 38.8 2.5 42.1 5.2 82.1 18.2 119 38.7 38.1 21.2 71.2 49.7 98.4 84.3 27.1 34.7 46.7 73.7 58.1 115.8a325.95 325.95 0 016.5 140.9h74.9c14.8-103.6-11.3-213-81-302.3z"></path>
              </svg>
            </span>
          </div>

          {/* 顺时针旋转 */}
          <div
            className="operation rotate-right"
            onClick={() => {
              setRotate(rotate + 90)
            }}
          >
            <span className="action">
              <svg
                className="svg-icon"
                viewBox="64 64 896 896"
                focusable="false"
                data-icon="rotate-right"
                fill="currentColor"
                aria-hidden="true"
              >
                <defs>
                  <style></style>
                </defs>
                <path d="M480.5 251.2c13-1.6 25.9-2.4 38.8-2.5v63.9c0 6.5 7.5 10.1 12.6 6.1L660 217.6c4-3.2 4-9.2 0-12.3l-128-101c-5.1-4-12.6-.4-12.6 6.1l-.2 64c-118.6.5-235.8 53.4-314.6 154.2A399.75 399.75 0 00123.5 631h74.9c-.9-5.3-1.7-10.7-2.4-16.1-5.1-42.1-2.1-84.1 8.9-124.8 11.4-42.2 31-81.1 58.1-115.8 27.2-34.7 60.3-63.2 98.4-84.3 37-20.6 76.9-33.6 119.1-38.8z"></path>
                <path d="M880 418H352c-17.7 0-32 14.3-32 32v414c0 17.7 14.3 32 32 32h528c17.7 0 32-14.3 32-32V450c0-17.7-14.3-32-32-32zm-44 402H396V494h440v326z"></path>
              </svg>
            </span>
          </div>

          {/* 缩小 */}
          <div
            className={`operation zoom-out ${minRotate ? 'disabled' : ''}`}
            onClick={() => {
              if (minRotate) return
              setMaxRotate(false)
              let hasFScaleX = scaleX < 0
              let hasFScaleY = scaleY < 0
              let scaleMin = scale - scaleDefault * SCALE_STEP
              if (scaleMin <= MIN_SCALE) {
                scaleMin = MIN_SCALE
                setScale(scaleMin)
                setMinRotate(true)
                setMaxRotate(false)
                setScaleX(hasFScaleX ? -scaleDefault : scaleDefault)
                setScaleY(hasFScaleY ? -scaleDefault : scaleDefault)
                return
              }

              setScale(scaleMin)
              let x = Math.abs(scaleX) - scaleDefault * SCALE_STEP
              let y = Math.abs(scaleY) - scaleDefault * SCALE_STEP
              setScaleX(hasFScaleX ? -x : x)
              setScaleY(hasFScaleY ? -y : y)
            }}
          >
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
          <div
            className={`operation zoom-in ${maxRotate ? 'disabled' : ''}`}
            onClick={() => {
              if (maxRotate) return
              setMinRotate(false)

              let hasFScaleX = scaleX < 0
              let hasFScaleY = scaleY < 0

              let scaleMax = scale + scaleDefault * SCALE_STEP
              if (scaleMax >= MAX_SCALE) {
                scaleMax = MAX_SCALE
                setMaxRotate(true)
                setMinRotate(false)
              }
              setScale(scaleMax)
              let x = Math.abs(scaleX) + scaleDefault * SCALE_STEP
              let y = Math.abs(scaleY) + scaleDefault * SCALE_STEP
              setScaleX(hasFScaleX ? -x : x)
              setScaleY(hasFScaleY ? -y : y)
            }}
          >
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
        </div>
      </div>
    )
  }

  const render = () => {
    return (
      <div className="image-wrapper flex-center wh100">
        <img
          src={props.content || ''}
          alt=""
          style={{
            transform: `translate3d(0px, 0px, 0px) scale3d(${scaleX}, ${scaleY}, 1) rotate(${rotate}deg)`,
          }}
        />
        {getActionsHtml()}
      </div>
    )
  }

  return render()
}

export default observer(Image)
