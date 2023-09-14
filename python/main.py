from djitellopy import Tello
import cv2
import pygame
import numpy as np
import time

# Speed of the drone
# 无人机的速度
S = 100
# Frames per second of the pygame window display
# A low number also results in input lag, as input information is processed once per frame.
# pygame窗口显示的帧数
# 较低的帧数会导致输入延迟，因为一帧只会处理一次输入信息
FPS = 120


class FrontEnd(object):
    """ Maintains the Tello display and moves it through the keyboard keys.
        Press escape key to quit.
        The controls are:
            - T: Takeoff
            - L: Land
            - Arrow keys: Forward, backward, left and right.
            - A and D: Counter clockwise and clockwise rotations (yaw)
            - W and S: Up and down.

        保持Tello画面显示并用键盘移动它
        按下ESC键退出
        操作说明：
            T：起飞
            L：降落
            方向键：前后左右
            A和D：逆时针与顺时针转向
            W和S：上升与下降

    """

    def __init__(self):
        # Init pygame
        # 初始化pygame
        pygame.init()
        pygame.joystick.init()

        

        # Creat pygame window
        # 创建pygame窗口
        pygame.display.set_caption("Tello video stream")
        self.screen = pygame.display.set_mode([1280, 800])

        # Init Tello object that interacts with the Tello drone
        # 初始化与Tello交互的Tello对象
        self.tello = Tello()

        # Drone velocities between -100~100
        # 无人机各方向速度在-100~100之间
        self.for_back_velocity = 0
        self.left_right_velocity = 0
        self.up_down_velocity = 0
        self.yaw_velocity = 0
        self.speed = 50

        self.send_rc_control = False

        # create update timer
        # 创建上传定时器
        pygame.time.set_timer(pygame.USEREVENT + 1, 1000 // FPS)

    def run(self):
        pygame.joystick.init()
        
        num_joysticks = pygame.joystick.get_count()

        should_stop = False
        if num_joysticks > 0:
            joystick = pygame.joystick.Joystick(0)
            joystick.init()
            print(f"Using {joystick.get_name()} gamepad")
        else:
            print("No gamepads detected")
        self.tello.connect()
        self.tello.set_speed(self.speed)
       # self.tello.set_video_resolution(Tello.RESOLUTION_720P)

        # In case streaming is on. This happens when we quit this program without the escape key.
        # 防止视频流已开启。这会在不使用ESC键退出的情况下发生。
        self.tello.streamoff()
        self.tello.streamon()

        frame_read = self.tello.get_frame_read()
        
        # Set Fullscren

        #pygame.display.toggle_fullscreen()

        while not should_stop:

            for event in pygame.event.get():
                if event.type == pygame.USEREVENT + 1:
                    self.update()
                elif event.type == pygame.QUIT:
                    should_stop = True
                elif event.type == pygame.KEYDOWN:
                    if event.key == pygame.K_ESCAPE:
                        should_stop = True
                    else:
                        self.keydown(event.key)
                elif event.type == pygame.KEYUP:
                    self.keyup(event.key)
                    
            axis_x = joystick.get_axis(3)
            axis_y = joystick.get_axis(4)
            axis_z = joystick.get_axis(1)
            axis_r = joystick.get_axis(0)
          
            self.left_right_velocity = int(axis_x * S * 1.5)
            self.for_back_velocity = int(axis_y * S * -1)
            self.up_down_velocity = int(axis_z * S * -1)
            self.yaw_velocity = int(axis_r * S * 1.5)
            
            print(f"Axis X: {axis_x}, Axis Y: {axis_y}")

            if frame_read.stopped:
                break

            self.screen.fill([0, 0, 0])

            frame = frame_read.frame
            # battery n. 电池
            text = "Battery: {}%".format(self.tello.get_battery())
            cv2.putText(frame, text, (5, 720 - 5),
                cv2.FONT_HERSHEY_COMPLEX, 1, (0, 0, 255), 2)
            
            text_lr = "L/R: {}%".format(self.left_right_velocity)
            cv2.putText(frame, text_lr, (5, 720 - 50),
                cv2.FONT_HERSHEY_COMPLEX, 1, (0, 0, 255), 2)
            
            text_fb = "F/B: {}%".format(self.for_back_velocity)
            cv2.putText(frame, text_fb, (300, 720 - 5),
                cv2.FONT_HERSHEY_COMPLEX, 1, (0, 0, 255), 2)
            
            text_ud = "U/D: {}%".format(self.up_down_velocity)
            cv2.putText(frame, text_ud, (300, 720 - 50),
                cv2.FONT_HERSHEY_COMPLEX, 1, (0, 0, 255), 2)
            
            text_yv = "Yaw: {}%".format(self.yaw_velocity)
            cv2.putText(frame, text_yv, (500, 720 - 5),
                cv2.FONT_HERSHEY_COMPLEX, 1, (0, 0, 255), 2)
            

            frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            frame = np.rot90(frame)
            frame = np.flipud(frame)

            #Resize
            frame = cv2.resize(frame, (1280, 800)) 

            frame = pygame.surfarray.make_surface(frame)
            self.screen.blit(frame, (0, 0))
            pygame.display.update()

            if joystick.get_button(3):
                self.tello.takeoff()
                self.send_rc_control = True
            elif joystick.get_button(2):
                not self.tello.land()
                self.send_rc_control = False
            

            # set full screen
            # 设置全屏

            

            time.sleep(1 / FPS)

        # Call it always before finishing. To deallocate resources.
        # 通常在结束前调用它以释放资源
        self.tello.end()

    def keydown(self, key):
        """ Update velocities based on key pressed
        Arguments:
            key: pygame key

        基于键的按下上传各个方向的速度
        参数：
            key：pygame事件循环中的键事件
        """
        if key == pygame.K_UP:  # set forward velocity
            self.for_back_velocity = S
        elif key == pygame.K_DOWN:  # set backward velocity
            self.for_back_velocity = -S
        elif key == pygame.K_LEFT:  # set left velocity
            self.left_right_velocity = -S
        elif key == pygame.K_RIGHT:  # set right velocity
            self.left_right_velocity = S
        elif key == pygame.K_w:  # set up velocity
            self.up_down_velocity = S
        elif key == pygame.K_s:  # set down velocity
            self.up_down_velocity = -S
        elif key == pygame.K_a:  # set yaw counter clockwise velocity
            self.yaw_velocity = -S
        elif key == pygame.K_d:  # set yaw clockwise velocity
            self.yaw_velocity = S

    def keyup(self, key):
        """ Update velocities based on key released
        Arguments:
            key: pygame key

        基于键的松开上传各个方向的速度
        参数：
            key：pygame事件循环中的键事件
       
        if key == pygame.K_UP or key == pygame.K_DOWN:  # set zero forward/backward velocity
           # self.for_back_velocity = 0
        elif key == pygame.K_LEFT or key == pygame.K_RIGHT:  # set zero left/right velocity
           #self.left_right_velocity = 0
        elif key == pygame.K_w or key == pygame.K_s:  # set zero up/down velocity
           # self.up_down_velocity = 0
        elif key == pygame.K_a or key == pygame.K_d:  # set zero yaw velocity
           # self.yaw_velocity = 0
         """
        if key == pygame.K_t:  # takeoff
            self.tello.takeoff()
            self.send_rc_control = True
        elif key == pygame.K_l:  # land
            not self.tello.land()
            self.send_rc_control = False
            

    def update(self):
        """ Update routine. Send velocities to Tello.

            向Tello发送各方向速度信息
        """
        
        if self.send_rc_control:
            self.tello.send_rc_control(self.left_right_velocity, self.for_back_velocity,
                self.up_down_velocity, self.yaw_velocity)


def main():
    frontend = FrontEnd()

    # run frontend

    frontend.run()


if __name__ == '__main__':
    main()