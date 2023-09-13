import pygame

pygame.init()
pygame.joystick.init()

num_joysticks = pygame.joystick.get_count()

if num_joysticks > 0:
    joystick = pygame.joystick.Joystick(0)
    joystick.init()
    print(f"Using {joystick.get_name()} gamepad")
else:
    print("No gamepads detected")

running = True
while running:
    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False

    axis_x = joystick.get_axis(0)
    axis_y = joystick.get_axis(1)
    trigger = joystick.get_axis(2)
    button_a = joystick.get_button(0)
    hat_state = joystick.get_hat(0)

    print(f"Axis X: {axis_x}, Axis Y: {axis_y}, Trigger: {trigger}, Button A: {button_a}, Hat: {hat_state}")

pygame.quit()
