/**
 * @fileOverview global loading
 * @date 2023-04-12
 * @author poohlaha
 */
import React, { ReactElement } from 'react'

interface ILoadingProps {
  show: boolean
}
const GlobalLoading: React.FC<ILoadingProps> = (props: ILoadingProps): ReactElement | null => {
  const render = () => {
    if (!props.show) return null
    return (
      <div className="loading">
        <div className="mask" />
        <div className="load">
          <div className="dot white" />
          <div className="dot" />
          <div className="dot" />
          <div className="dot" />
          <div className="dot" />
        </div>
      </div>
    )
  }

  return render()
}

export default GlobalLoading
