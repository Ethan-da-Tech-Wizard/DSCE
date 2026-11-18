# Program: DSCE - gameboy_app.py
# Programmer: Ethan-da-Tech-Wizard (written with Claude, an AI assistant)
# Last Modified: 2026-11-18
# Purpose: Tkinter desktop app combining a small paddle-and-ball arcade game with a limited-range calculator in one window.
# assembled by the DSCE polyglot assembler (Python)
import tkinter as tk
import tkinter.messagebox as messagebox

# --- CONFIGURATION & PARAMETERS ---
BALL_COLOR = 'red'
BALL_SPEED = 5
CALC_LIMIT = 100

# --- APP SETUP ---
root = tk.Tk()
root.title('GameBoy Hybrid Console')
root.geometry('450x650')
root.configure(bg='#2c2c2c')

# ==========================================
# PART 1: PLAYABLE PADDLE GAME
# ==========================================
game_frame = tk.LabelFrame(root, text='Arcade: Bounce Game', bg='#2c2c2c', fg='white', font=('Arial', 12, 'bold'))
game_frame.pack(pady=10, padx=10, fill='both', expand=True)

canvas = tk.Canvas(game_frame, width=400, height=250, bg='black', highlightthickness=0)
canvas.pack(pady=5)

ball = canvas.create_oval(190, 40, 210, 60, fill=BALL_COLOR)
paddle = canvas.create_rectangle(160, 230, 240, 240, fill='white')
score = 0
score_label = tk.Label(game_frame, text='Score: 0', bg='#2c2c2c', fg='white', font=('Arial', 10, 'bold'))
score_label.pack()

ball_x_speed = BALL_SPEED
ball_y_speed = BALL_SPEED
paddle_speed = 20

def move_left(event):
    """Key handler: slide the paddle left, clamping it to the canvas edge."""
    canvas.move(paddle, -paddle_speed, 0)
    pos = canvas.coords(paddle)
    if pos[0] < 0:
        canvas.coords(paddle, 0, 230, 80, 240)

def move_right(event):
    """Key handler: slide the paddle right, clamping it to the canvas edge."""
    canvas.move(paddle, paddle_speed, 0)
    pos = canvas.coords(paddle)
    if pos[2] > 400:
        canvas.coords(paddle, 320, 230, 400, 240)

root.bind('<Left>', move_left)
root.bind('<Right>', move_right)

def game_loop():
    """Advance the ball one frame, handle bounces, scoring and reset, then reschedule itself."""
    global ball_x_speed, ball_y_speed, score
    canvas.move(ball, ball_x_speed, ball_y_speed)
    ball_pos = canvas.coords(ball)
    paddle_pos = canvas.coords(paddle)

    # Bounce off the left/right walls and the top wall.
    if ball_pos[0] <= 0 or ball_pos[2] >= 400:
        ball_x_speed = -ball_x_speed
    if ball_pos[1] <= 0:
        ball_y_speed = -ball_y_speed

    # Ball reaches the paddle row inside its width (with 10px slack): bounce up and score.
    if (ball_pos[3] >= paddle_pos[1] and
        ball_pos[0] >= paddle_pos[0] - 10 and
        ball_pos[2] <= paddle_pos[2] + 10):
        ball_y_speed = -abs(ball_y_speed)
        score += 1
        score_label.config(text=f'Score: {score}')

    # Ball fell past the bottom: put it back at the start and reset the score.
    if ball_pos[3] >= 250:
        canvas.coords(ball, 190, 40, 210, 60)
        ball_y_speed = -abs(ball_y_speed)
        score = 0
        score_label.config(text='Game Over! Score Reset to 0')

    root.after(30, game_loop)  # run again in 30 ms (about 33 frames per second)

# ==========================================
# PART 2: INTERACTIVE CALCULATOR
# ==========================================
calc_frame = tk.LabelFrame(root, text='Calculator Console', bg='#2c2c2c', fg='white', font=('Arial', 12, 'bold'))
calc_frame.pack(pady=10, padx=10, fill='both', expand=True)

display = tk.Entry(calc_frame, font=('Arial', 16), justify='right', bd=5, bg='#1e1e1e', fg='white', insertbackground='white')
display.pack(fill='x', padx=10, pady=5)

def press_key(key):
    """Handle a calculator button: '=' evaluates the display, 'C' clears it, anything else is appended."""
    if key == '=':
        try:
            val = eval(display.get())
            if float(val) > float(CALC_LIMIT):
                display.delete(0, tk.END)
                display.insert(tk.END, 'OVER LIMIT!')
            else:
                display.delete(0, tk.END)
                display.insert(tk.END, str(val))
        except Exception:
            display.delete(0, tk.END)
            display.insert(tk.END, 'ERROR')
    elif key == 'C':
        display.delete(0, tk.END)
    else:
        display.insert(tk.END, key)

# Calculator keypad layout, one inner list per row of buttons.
buttons_layout = [
    ['7', '8', '9', '/'],
    ['4', '5', '6', '*'],
    ['1', '2', '3', '-'],
    ['C', '0', '=', '+']
]

grid_frame = tk.Frame(calc_frame, bg='#2c2c2c')
grid_frame.pack()

# Create one button per key; the default argument v=val freezes each key for its callback.
for r_idx, row in enumerate(buttons_layout):
    for c_idx, val in enumerate(row):
        btn = tk.Button(grid_frame, text=val, font=('Arial', 12, 'bold'), width=5, height=2,
                        bg='#4a4a4a', fg='white', activebackground='#6a6a6a', activeforeground='white',
                        command=lambda v=val: press_key(v))
        btn.grid(row=r_idx, column=c_idx, padx=5, pady=5)

game_loop()  # start the animation
root.mainloop()  # hand control to the Tk event loop
